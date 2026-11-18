//! The client.

use std::collections::HashMap;
use std::pin::Pin;

use bytes::Bytes;
use futures_util::{Stream, StreamExt as _, future};
use reqwest::{Response, StatusCode, header};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::host::image_manager::manifest::ACCEPT;

/// A blob's body, piece by piece; it ends short where the transfer
/// failed, which the reader's digest check sees.
pub type Body = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

/// The client: the registries the configuration lists, each by its
/// host, and the token each repository was last given. It holds no
/// credential, and reads from every registry anonymously; a publish
/// writes with the credential its request carries, through a
/// [`Writer`](super::Writer) of its own that shares nothing with these
/// tokens.
/// See [`registry_client`](super) for what it asks and how it
/// authenticates.
#[derive(Debug)]
pub struct RegistryClient {
    http: reqwest::Client,
    registries: Vec<String>,
    /// By `<host>/<name>`: the Bearer token its token service gave.
    tokens: Mutex<HashMap<String, String>>,
}

impl RegistryClient {
    /// A client of `registries`; one that cannot be made — no TLS on
    /// this host — is the reason.
    pub fn new(registries: &[String]) -> Result<Self, String> {
        let http = reqwest::Client::builder().build().map_err(|error| error.to_string())?;
        Ok(RegistryClient {
            http,
            registries: registries.to_vec(),
            tokens: Mutex::new(HashMap::new()),
        })
    }

    /// The HTTP client every request goes out on.
    pub(super) fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Whether the configuration lists `host`: a reference naming any
    /// other is never asked.
    pub fn lists(&self, host: &str) -> bool {
        self.registries.iter().any(|registry| registry == host)
    }

    /// The manifest under `digest` in `<host>/<name>`, whole; `None`
    /// where the registry answers `404`.
    pub async fn manifest(&self, host: &str, name: &str, digest: &str) -> Result<Option<Bytes>, String> {
        let Some(response) = self.get(host, name, &format!("manifests/{digest}"), Some(ACCEPT)).await? else {
            return Ok(None);
        };
        response.bytes().await.map(Some).map_err(|error| error.to_string())
    }

    /// The blob under `digest` in `<host>/<name>`, piece by piece;
    /// `None` where the registry answers `404`.
    pub async fn blob(&self, host: &str, name: &str, digest: &str) -> Result<Option<Body>, String> {
        let Some(response) = self.get(host, name, &format!("blobs/{digest}"), None).await? else {
            return Ok(None);
        };
        let body = response
            .bytes_stream()
            .take_while(|piece| future::ready(piece.is_ok()))
            .filter_map(|piece| future::ready(piece.ok()));
        Ok(Some(Box::pin(body)))
    }

    /// One `GET` of `<path>` under the repository, with the token the
    /// repository was last given, if any, and once more after a
    /// `401`'s challenge is answered. `None` for a `404`.
    async fn get(&self, host: &str, name: &str, path: &str, accept: Option<&str>) -> Result<Option<Response>, String> {
        if !self.lists(host) {
            return Err(format!("the registry `{host}` is not one the provider uses"));
        }
        let (api, name) = api(host, name);
        let url = format!("https://{api}/v2/{name}/{path}");
        let key = format!("{host}/{name}");
        let token = self.tokens.lock().await.get(&key).cloned();
        let response = self.send(&url, accept, token.as_deref()).await?;
        let response = if response.status() == StatusCode::UNAUTHORIZED {
            let challenge = response
                .headers()
                .get(header::WWW_AUTHENTICATE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_string();
            let token = self.answer(&challenge).await?;
            self.tokens.lock().await.insert(key, token.clone());
            self.send(&url, accept, Some(&token)).await?
        } else {
            response
        };
        match response.status() {
            StatusCode::OK => Ok(Some(response)),
            StatusCode::NOT_FOUND => Ok(None),
            status => Err(format!("`{url}` answered {status}")),
        }
    }

    /// The request sent, with `token` as its Bearer where there is
    /// one.
    async fn send(&self, url: &str, accept: Option<&str>, token: Option<&str>) -> Result<Response, String> {
        let mut request = self.http.get(url);
        if let Some(accept) = accept {
            request = request.header(header::ACCEPT, accept);
        }
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        request.send().await.map_err(|error| error.to_string())
    }

    /// A `401`'s challenge answered: `Bearer`, with a token the realm
    /// gives an anonymous asker. A `Basic` challenge asks for a
    /// credential, and the provider holds none; that, and a challenge
    /// of any other scheme, is the error.
    async fn answer(&self, challenge: &str) -> Result<String, String> {
        let (scheme, parameters) = challenge.split_once(' ').unwrap_or((challenge, ""));
        if scheme.eq_ignore_ascii_case("basic") {
            return Err("the registry asks for a credential, and the provider holds none".to_string());
        }
        if !scheme.eq_ignore_ascii_case("bearer") {
            return Err(format!("the registry's challenge `{challenge}` is not one the provider answers"));
        }
        let parameters = parse(parameters);
        let realm = parameters.get("realm").ok_or("the registry's Bearer challenge names no realm")?;
        let mut query = Vec::new();
        for key in ["service", "scope"] {
            if let Some(value) = parameters.get(key) {
                query.push((key, value.as_str()));
            }
        }
        let response = self.http.get(realm.as_str()).query(&query).send().await.map_err(|error| error.to_string())?;
        if !response.status().is_success() {
            return Err(format!("the token service answered {}", response.status()));
        }
        let body = json(response).await?;
        body.get("token")
            .or_else(|| body.get("access_token"))
            .and_then(Value::as_str)
            .map(|token| token.to_string())
            .ok_or_else(|| "the token service answered no token".to_string())
    }
}

/// The JSON of a response, read whole.
pub(super) async fn json(response: Response) -> Result<Value, String> {
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

/// The host the API is reached at, and the name as that host knows
/// it: Docker Hub's API host, and `library/` before a one-segment
/// name there.
pub(super) fn api(host: &str, name: &str) -> (String, String) {
    if host == "docker.io" {
        let name = if name.contains('/') { name.to_string() } else { format!("library/{name}") };
        ("registry-1.docker.io".to_string(), name)
    } else {
        (host.to_string(), name.to_string())
    }
}

/// A challenge's parameters, `key="value"` or `key=value`, separated
/// by commas; a comma inside quotes is the value's.
pub(super) fn parse(parameters: &str) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    let mut rest = parameters.trim();
    while !rest.is_empty() {
        let Some((key, after)) = rest.split_once('=') else {
            break;
        };
        let key = key.trim().trim_start_matches(',').trim().to_ascii_lowercase();
        let (value, after) = if let Some(quoted) = after.strip_prefix('"') {
            match quoted.find('"') {
                Some(end) => (&quoted[..end], &quoted[end + 1..]),
                None => (quoted, ""),
            }
        } else {
            match after.find(',') {
                Some(end) => (&after[..end], &after[end..]),
                None => (after, ""),
            }
        };
        parsed.insert(key, value.to_string());
        rest = after.trim_start().trim_start_matches(',').trim_start();
    }
    parsed
}
