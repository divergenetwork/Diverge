//! AWS Bedrock.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// AWS Bedrock — AWS credentials, not a provider key.
///
/// APPLICATION: the harness sets the AWS SDK's own env vars in the
/// gateway's process environment before Hermes starts, each to the
/// member of [`auth`](Self::auth) that carries it — see [`Auth`] for
/// which — plus `AWS_REGION` to
/// [`region`](Self::region) when present. These must be REAL
/// process env: the botocore chain underneath never reads Hermes's
/// dotenv.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Provider {
    /// The discriminator. Always `bedrock`.
    pub provider: Bedrock,
    /// The credential, in one of AWS's static shapes. See [`Auth`].
    pub auth: Auth,
    /// The region, applied as `AWS_REGION`; absent = Hermes's own
    /// default (`us-east-1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// One of AWS's static credential shapes: an object whose `type` names
/// the shape, in kebab-case, and whose other members are that shape's
/// credentials, every one required.
///
/// The rest of the botocore chain (profiles, container/IMDS roles,
/// web identity) is deliberately absent: those are facts about a
/// machine, and a request has no machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Auth {
    /// A Bedrock API key — the first credential the chain checks.
    BearerToken {
        /// The key, applied as `AWS_BEARER_TOKEN_BEDROCK`.
        bearer_token: String,
    },
    /// Static SigV4 credentials.
    AccessKey {
        /// The access key's id, applied as `AWS_ACCESS_KEY_ID`.
        access_key_id: String,
        /// The secret access key, applied as `AWS_SECRET_ACCESS_KEY`.
        secret_access_key: String,
    },
    /// Temporary SigV4 credentials.
    SessionToken {
        /// The access key's id, applied as `AWS_ACCESS_KEY_ID`.
        access_key_id: String,
        /// The secret access key, applied as `AWS_SECRET_ACCESS_KEY`.
        secret_access_key: String,
        /// The session token, applied as `AWS_SESSION_TOKEN`.
        session_token: String,
    },
}

/// [`Provider`]'s discriminator.
///
/// One variant, and the reason the [union](super::Provider) can be
/// untagged: no other provider's arguments can produce this value.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Bedrock {
    #[default]
    Bedrock,
}
