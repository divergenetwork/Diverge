//! A request's body, and any other JSON the command line carries, as
//! its own type.

use serde::de::DeserializeOwned;

use super::Error;

/// The body `text` carries, decoded as the request's own type —
/// `{}` when none was given, so a request whose every member has a
/// default takes none. A body the request does not take is
/// [`Error::Body`], with the path to the member that was wrong.
pub fn body<Q: DeserializeOwned>(text: Option<&str>) -> Result<Q, Error> {
    json(text.unwrap_or("{}")).map_err(Error::Body)
}

/// `text` decoded as `T`, the failure naming the path to the member
/// that was wrong: what [`body`] is made of, for the other JSON a leaf
/// takes.
pub fn json<T: DeserializeOwned>(
    text: &str,
) -> Result<T, serde_path_to_error::Error<serde_json::Error>> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    serde_path_to_error::deserialize(&mut deserializer)
}

/// [`body`], with `extra` added to the list its member `member` is:
/// what a repeatable flag gives beside the body, for a member the
/// request requires. The list is made where the body has none, so a
/// request is whole with the flag alone; a body that is not an object
/// is refused as the request refuses it.
pub fn body_with<Q: DeserializeOwned>(text: Option<&str>, member: &str, extra: Vec<String>) -> Result<Q, Error> {
    let mut value: serde_json::Value = json(text.unwrap_or("{}")).map_err(Error::Body)?;
    if !extra.is_empty() {
        if let Some(object) = value.as_object_mut() {
            let list = object.entry(member).or_insert_with(|| serde_json::Value::Array(Vec::new()));
            if let Some(list) = list.as_array_mut() {
                list.extend(extra.into_iter().map(serde_json::Value::String));
            }
        }
    }
    serde_path_to_error::deserialize(value).map_err(Error::Body)
}
