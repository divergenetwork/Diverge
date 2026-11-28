//! Who is asking: the person behind a frontend request, the agent
//! behind an API request — each as the daemon attests it, never as
//! the request says.

use axum::http::HeaderMap;
use diverge_sdk::shared::Identity;
use diverge_sdk::daemon::key;
use diverge_sdk::shared::mcp::attestation::Caller;

/// The person behind a page's request, from the `diverge.network`
/// header the daemon stamped it with: the account. A request with no
/// stamp, one that does not read, or one stamped as an agent, is
/// refused, with why.
pub fn person(headers: &HeaderMap) -> Result<Identity, String> {
    let stamp = headers.get(diverge_sdk::shared::mcp::KEY).ok_or_else(|| "the request carries no stamp".to_string())?;
    let text = stamp.to_str().map_err(|error| error.to_string())?;
    match serde_json::from_str::<Caller>(text) {
        Ok(Caller::Client { identity, .. }) => Ok(identity),
        Ok(Caller::Agent(_)) => Err("an agent, not a person".to_string()),
        Ok(Caller::Tool { .. }) => Err("a tool, not a person".to_string()),
        Err(error) => Err(error.to_string()),
    }
}

/// The agent behind an API request, from the same stamp: its key. A
/// request with no stamp, one that does not read, or one stamped as a
/// client or as a tool, is refused, with why: the desk's API is its
/// agent's alone.
pub fn agent(headers: &HeaderMap) -> Result<key::Agent, String> {
    let stamp = headers.get(diverge_sdk::shared::mcp::KEY).ok_or_else(|| "the request carries no stamp".to_string())?;
    let text = stamp.to_str().map_err(|error| error.to_string())?;
    match serde_json::from_str::<Caller>(text) {
        Ok(Caller::Agent(agent)) => Ok(agent.key),
        Ok(Caller::Client { .. }) => Err("a client, not the desk's agent".to_string()),
        Ok(Caller::Tool { .. }) => Err("a tool, not the desk's agent".to_string()),
        Err(error) => Err(error.to_string()),
    }
}
