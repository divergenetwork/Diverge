//! `list_accounts`: the accounts the tool holds.

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::failure::Failure;
use crate::server::Call;
use crate::store;

/// The tool's name.
pub const NAME: &str = "list_accounts";

/// What the tool does, as its caller reads it.
pub const DESCRIPTION: &str = "List the accounts at identity.diverge.network this tool holds: `{\"accounts\": [{\"username\", \"account\", \"issuer\", \"id\"}]}`. \
An account is a GitHub or Google sign-in, and its `username` is what it is named by: every subject it owns is `<username>.<name>`, and its registry namespace is `<username>`. \
Give a `username` to `login`, or to `publish`. An account whose `username` is null has set none and can do neither: a person sets it, once and for good, at this tool's page. \
No token is answered. Accounts are added by a person at this tool's page, never by a tool call.";

/// What `list_accounts` takes: nothing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {}

/// Every account kept under the tool's template, by issuer and then
/// id: `{"accounts": [{"username", "account", "issuer", "id"}]}` —
/// see [`list`](store::accounts::list), which asks
/// `identity.diverge.network` the username of each account it has not
/// learned one of yet. None is an empty list, and no failure.
pub async fn call(call: Call<'_>, _parameters: Parameters) -> Result<Value, Failure> {
    let pool = call.state.store.pool(&call.state.client).await?;
    let accounts = store::accounts::list(pool, &call.state.http, call.template).await?;
    Ok(json!({ "accounts": accounts }))
}
