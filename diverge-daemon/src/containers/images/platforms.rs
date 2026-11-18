//! The platforms an image's manifest names, read through a source.

use diverge_sdk::shared::platform;
use futures_util::StreamExt as _;
use serde_json::Value;

use super::Source;

/// Every platform the manifest under `digest` names, each once, in
/// its order, read through `source`; `None` where the source does not
/// supply the manifest, or it states no platform. The rules:
///
/// 1. **An index.** Each entry's platform, as
///    [`platform::of`] names it: its `os`, its `architecture` and,
///    where it has one, its `variant`. An entry that states no `os` or
///    no `architecture` is for `unknown` in its place, as a provider
///    names it — an attestation is such an entry, and is one of the
///    platforms the manifest names all the same.
/// 2. **An image that is not an index.** The one platform its config
///    states, the config read through the source as a blob is.
pub async fn platforms(source: &Source, digest: &str) -> Option<Vec<String>> {
    let manifest = source.scope.manifest(digest).await.ok()??;
    let value: Value = serde_json::from_slice(&manifest.body).ok()?;
    // Rule 1.
    if let Some(entries) = value.get("manifests").and_then(Value::as_array) {
        let mut named: Vec<String> = Vec::new();
        for entry in entries {
            let stated = stated(entry.get("platform").unwrap_or(&Value::Null));
            if !named.contains(&stated) {
                named.push(stated);
            }
        }
        return (!named.is_empty()).then_some(named);
    }
    // Rule 2.
    let config = value.get("config")?.get("digest")?.as_str()?;
    let mut pieces = source.scope.blob(config).await.ok()?;
    let mut body = Vec::new();
    while let Some(piece) = pieces.next().await {
        body.extend_from_slice(&piece.ok()?);
    }
    let config: Value = serde_json::from_slice(&body).ok()?;
    config.get("os")?.as_str()?;
    config.get("architecture")?.as_str()?;
    Some(vec![stated(&config)])
}

/// The platform `value` states, by its `os`, `architecture` and
/// `variant` members, `unknown` standing for one of the first two it
/// does not state.
fn stated(value: &Value) -> String {
    let field = |name: &str| value.get(name).and_then(Value::as_str);
    platform::of(field("os").unwrap_or("unknown"), field("architecture").unwrap_or("unknown"), field("variant"))
}
