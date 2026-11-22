//! `publish`: publish images to Diverge's registry, as an account.

use diverge_sdk::daemon::agent_log::Identity;
use diverge_sdk::daemon::endpoints::images::publish::{client::execute, client::request, server::response};
use diverge_sdk::identity_diverge_network::{self, REGISTRY};
use diverge_sdk::shared::containers::request::Reference;
use diverge_sdk::shared::description::Description;
use futures_util::StreamExt as _;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::failure::Failure;
use crate::server::Call;
use crate::store;

/// The tool's name.
pub const NAME: &str = "publish";

/// What the tool does, as its caller reads it.
pub const DESCRIPTION: &str = "Publish one or more images to Diverge's registry, registry.diverge.network, as one of this tool's accounts, and wait for it. \
Give the account's `issuer` and `id` as `list_accounts` gives them, a `repository` name, and the `images` by digest: the image goes to `<username>/<repository>` there, the username the account's own, set once at identity.diverge.network. \
No username, password or registry is given: the tool supplies the account's own. The daemon gathers the images from wherever they are, merges several into one, and publishes. \
Answers `{\"published\": {\"provider\", \"digest\", \"created\", \"reference\", \"tags\"}}`.";

/// What `publish` takes.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    /// The account's issuer, as `list_accounts` gives it: `github` or
    /// `google`.
    pub issuer: String,
    /// The account's id at that issuer, as `list_accounts` gives it.
    pub id: String,
    /// The repository's name in the account's namespace: one name,
    /// with no `/`, of lowercase letters and digits with single `.`,
    /// `_` or `-` between them. The image goes to
    /// `registry.diverge.network/<username>/<repository>`, the
    /// username the account's own.
    pub repository: String,
    /// The images to publish as one, each a digest and, optionally,
    /// the platforms to take of it: one or more. One image taking all
    /// its platforms is published as it is; anything else is merged
    /// into one new image first.
    pub images: Vec<request::Part>,
    /// The tags to put on it. Absent or empty, the image is published
    /// by its digest alone.
    #[serde(default)]
    pub tags: Vec<String>,
    /// What an image this publish makes is kept as, on the provider it
    /// is done on: one short line. Not used where the image was there
    /// already.
    #[serde(default)]
    pub description: Option<Description>,
    /// The provider to do it on. Absent, the daemon picks one that
    /// uses the registry.
    #[serde(default)]
    pub provider: Option<Identity>,
}

/// Publish `images` to `<username>/<repository>` at [`REGISTRY`].
/// The rules:
///
/// 1. **The account is one the tool holds.** Its issuer is one there
///    is, else [`Failure::Issuer`]; it is kept under the tool's
///    template, else [`Failure::NoAccount`]. Neither the id nor the
///    repository is empty, the repository is one name, and at least
///    one image is named.
/// 2. **Where it goes is not the caller's to say.** The registry is
///    [`REGISTRY`], and the repository's namespace is the account's
///    username — see [`username`](store::accounts::username) — the
///    one namespace the account may write. An account that has set
///    none has no namespace: [`Failure::NoUsername`].
/// 3. **Its token is had first** — see
///    [`token`](store::accounts::token) — the one the registry takes
///    of its issuer, which is the one `identity.diverge.network`
///    takes: under the account's row lock, renewed where it has
///    expired. An account that cannot write begins no publish.
/// 4. **The credential is the account's.** The username is the
///    issuer's name and the password the token, as the registry takes
///    them. It is handed to the daemon with the request and answered
///    to nobody: no answer of this tool carries it.
/// 5. **The daemon publishes**: its `images publish`, which picks a
///    provider, brings onto it what it lacks, merges where there is
///    more than one image, and has the provider send the result. The
///    tool is served under its own grants, and a daemon that grants
///    it no publish answers [`Failure::Unpublished`] saying so.
/// 6. **Waited for.** The answers are read to the one that ends the
///    publish. `published` is the tool's answer, with the image
///    merged beside it where one was. Any other ending is the failure,
///    by its own name.
pub async fn call(call: Call<'_>, parameters: Parameters) -> Result<Value, Failure> {
    // Rule 1.
    let issuer = identity_diverge_network::issuer(&parameters.issuer).map_err(|error| Failure::Issuer(error.to_string()))?;
    if parameters.id.is_empty() || parameters.repository.is_empty() {
        return Err(Failure::Parameters("neither `id` nor `repository` may be empty".to_string()));
    }
    if parameters.repository.contains('/') {
        return Err(Failure::Parameters("`repository` is one name, with no `/`: the account is its namespace".to_string()));
    }
    if parameters.images.is_empty() {
        return Err(Failure::Parameters("`images` names one or more images".to_string()));
    }
    // Rule 3.
    let pool = call.state.store.pool(&call.state.client).await?;
    let token = store::accounts::token(pool, &call.state.http, call.template, issuer, &parameters.id, issuer.presents).await?;
    let username = store::accounts::username(&call.state.http, issuer, &parameters.id, &token).await?;
    // Rules 2 and 4.
    let request = request::Frame {
        images: parameters.images,
        reference: Reference {
            registry: REGISTRY.to_string(),
            name: format!("{username}/{}", parameters.repository),
        },
        tags: parameters.tags,
        credential: Some(request::Credential {
            username: issuer.name.to_string(),
            password: token,
        }),
        description: parameters.description,
        provider: parameters.provider,
        relay: None,
    };
    // Rule 5.
    let handle = call.state.daemon.handle(&call.state.client).await?;
    let mut answers = match execute::execute(&handle, &request).await {
        Ok(answers) => answers,
        Err(error) => {
            call.state.daemon.drop_handle().await;
            return Err(Failure::Unpublished(format!("the daemon could not be asked: {error}")));
        }
    };
    // Rule 6.
    let mut merged: Option<response::Merged> = None;
    while let Some(answer) = answers.next().await {
        let answer = answer.map_err(|error| Failure::Unpublished(format!("the daemon could not be asked: {error}")))?;
        let why = match answer {
            response::Frame::Blob(_) | response::Frame::Added(_) | response::Frame::Uploaded(_) => continue,
            response::Frame::Merged(made) => {
                merged = Some(made);
                continue;
            }
            response::Frame::Published(published) => {
                return Ok(match merged {
                    Some(merged) => json!({ "published": published, "merged": merged }),
                    None => json!({ "published": published }),
                });
            }
            response::Frame::Forbidden => "the daemon grants this tool no publish over a provider that could do it: it needs `{\"images\": {\"actions\": [\"publish\"], \"within\": \"any\"}}`".to_string(),
            response::Frame::NotFound => "the provider named is none the daemon keeps a record of".to_string(),
            response::Frame::NoProvider => format!("no provider the daemon reaches is connected, keeps a Store and uses the registry `{REGISTRY}`: a provider must list that registry among those it uses"),
            response::Frame::RegistryUnlisted => format!("the provider named does not use the registry `{REGISTRY}`"),
            response::Frame::Unsupported => "the provider keeps no Store".to_string(),
            response::Frame::Unavailable => "an image, or a platform of one, is held by no provider and no daemon the daemon reaches".to_string(),
            response::Frame::PlatformRepeated => "two of the images take the same platform: name `platforms` on each so that each platform is taken once".to_string(),
            response::Frame::PlatformNotFound => "a platform named is not one its image has".to_string(),
            response::Frame::InsufficientCapacity => "the provider's Store has no room".to_string(),
            response::Frame::ProofFailed => "a source supplied an image and did not prove it holds it".to_string(),
            response::Frame::Malformed => "an image is not one the provider takes, or the repository or a tag is not a name a registry takes".to_string(),
            response::Frame::Unauthorized => "the registry did not take the account's token".to_string(),
            response::Frame::Denied => "the account may not write to that repository".to_string(),
            response::Frame::Error(error) => error.0.to_string(),
        };
        return Err(Failure::Unpublished(why));
    }
    Err(Failure::Unpublished("the daemon's publish ended with no outcome".to_string()))
}
