//! `POST /register`: the arguments, and whose desk this is, once.

use axum::Json;
use axum::http::StatusCode;
use diverge_sdk::container_proxy::inside::register;
use diverge_sdk::container_proxy::inside::register::response::Response;
use diverge_sdk::daemon::key;
use diverge_sdk::shared::containers::request::Which;
use serde_json::{Value, json};
use tokio::sync::OnceCell;

use crate::arguments::Arguments;

/// The agent the desk is deployed for, kept at the one registration.
static OWNER: OnceCell<key::Agent> = OnceCell::const_new();

/// The agent the desk is deployed for; `None` before the registration.
pub fn owner() -> Option<&'static key::Agent> {
    OWNER.get()
}

/// `POST /register`. The rules:
///
/// 1. Arguments that are not [`Arguments`] are `400`.
/// 2. The desk is one agent's: a registration that says the tool is
///    owned by an agent keeps that agent; one that says anything else
///    — every agent's, or a record by index — is `400`, since the
///    image's scope is `instance` and a desk with no agent has nobody
///    to answer as.
/// 3. A second registration, whatever it carries, is `409`.
/// 4. Otherwise `200`, declaring no dependency and no permission.
pub async fn register(Json(request): Json<register::request::Request>) -> Result<(StatusCode, Json<Response>), (StatusCode, Json<Value>)> {
    // Rule 1.
    if let Err(error) = serde_json::from_value::<Arguments>(request.arguments) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "kind": "arguments", "error": error.to_string() }))));
    }
    // Rule 2.
    let Which::Owner(key::Owner::Agent(agent)) = request.which else {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "kind": "owner", "error": "the desk is one agent's, and this registration names none" }))));
    };
    // Rules 3 and 4.
    match OWNER.set(agent) {
        Ok(()) => Ok((StatusCode::OK, Json(Response::default()))),
        Err(_) => Err((StatusCode::CONFLICT, Json(json!({ "kind": "registered", "error": "the desk is registered, and it never changes" })))),
    }
}
