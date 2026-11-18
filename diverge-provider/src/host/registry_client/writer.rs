//! Writing to a registry: one publish's uploads.

use reqwest::{Body, RequestBuilder, Response, StatusCode, header};
use serde_json::Value;

use super::registry_client::{RegistryClient, api, json, parse};

/// How a write was refused, or failed.
#[derive(Debug)]
pub enum Refused {
    /// The registry, or its token service, did not take the
    /// credential, or wanted one and was given none.
    Unauthorized,
    /// The credential was taken, and it may not write to the
    /// repository.
    Denied,
    /// Anything else, in the registry's or the transport's words.
    Other(String),
}

/// What a publish presents, once it is known what the registry takes.
enum Auth {
    /// Nothing: the registry asked for nothing.
    None,
    /// The username and password, on every request.
    Basic(String, String),
    /// The token its token service gave for this repository.
    Bearer(String),
}

/// One publish's hold on one repository of a registry: the repository,
/// and what is presented to write to it. The rules:
///
/// 1. **Made for one publish, and dropped with it.** The credential
///    and the token got with it live here and nowhere else: not in
///    the client's cache, which other requests read, and on no disk.
/// 2. **Asked first.** [`RegistryClient::writer`] asks the registry's
///    root what it takes before anything is uploaded: nothing, the
///    credential as `Basic`, or a `Bearer` token, which its token
///    service is asked for with the credential and the scope
///    `repository:<name>:pull,push`. A body that streams cannot be
///    sent twice, so no upload waits on a challenge.
/// 3. **A `401` after that is the credential refused**, and a `403`,
///    or a `401` naming `insufficient_scope`, is the credential not
///    allowed to write there.
/// 4. **Blobs first, then manifests.** A registry takes a manifest
///    only once it holds what the manifest names; the order is the
///    caller's to keep.
pub struct Writer<'a> {
    client: &'a RegistryClient,
    /// The host the API is reached at.
    api: String,
    /// The repository, as that host knows it.
    name: String,
    auth: Auth,
}

impl RegistryClient {
    /// A writer on `<host>/<name>`, presenting `credential`: rule 2 of
    /// [`Writer`] done. A registry the configuration does not list is
    /// never asked: that is the caller's to have refused.
    pub async fn writer(&self, host: &str, name: &str, credential: Option<(&str, &str)>) -> Result<Writer<'_>, Refused> {
        let (api, name) = api(host, name);
        let response = self.http().get(format!("https://{api}/v2/")).send().await.map_err(other)?;
        let auth = match response.status() {
            StatusCode::UNAUTHORIZED => {
                let challenge = challenge(&response);
                let (scheme, parameters) = challenge.split_once(' ').unwrap_or((challenge.as_str(), ""));
                if scheme.eq_ignore_ascii_case("basic") {
                    match credential {
                        Some((username, password)) => Auth::Basic(username.to_string(), password.to_string()),
                        None => return Err(Refused::Unauthorized),
                    }
                } else if scheme.eq_ignore_ascii_case("bearer") {
                    Auth::Bearer(self.token(parameters, &name, credential).await?)
                } else {
                    return Err(Refused::Other(format!("the registry's challenge `{challenge}` is not one the provider answers")));
                }
            }
            status if status.is_success() => Auth::None,
            status => return Err(Refused::Other(format!("`https://{api}/v2/` answered {status}"))),
        };
        Ok(Writer { client: self, api, name, auth })
    }

    /// A token to pull from and push to `name`, asked of the realm a
    /// `Bearer` challenge names, as `credential` where there is one.
    async fn token(&self, parameters: &str, name: &str, credential: Option<(&str, &str)>) -> Result<String, Refused> {
        let parameters = parse(parameters);
        let realm = parameters.get("realm").ok_or_else(|| Refused::Other("the registry's Bearer challenge names no realm".to_string()))?;
        let scope = format!("repository:{name}:pull,push");
        let mut query = vec![("scope", scope.as_str())];
        if let Some(service) = parameters.get("service") {
            query.push(("service", service.as_str()));
        }
        let mut request = self.http().get(realm.as_str()).query(&query);
        if let Some((username, password)) = credential {
            request = request.basic_auth(username, Some(password));
        }
        let response = request.send().await.map_err(other)?;
        match response.status() {
            StatusCode::UNAUTHORIZED => return Err(Refused::Unauthorized),
            StatusCode::FORBIDDEN => return Err(Refused::Denied),
            status if !status.is_success() => return Err(Refused::Other(format!("the token service answered {status}"))),
            _ => {}
        }
        let body = json(response).await.map_err(Refused::Other)?;
        body.get("token")
            .or_else(|| body.get("access_token"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| Refused::Other("the token service answered no token".to_string()))
    }
}

impl Writer<'_> {
    /// Whether the repository holds the blob under `digest` already: a
    /// `HEAD` of it.
    pub async fn has_blob(&self, digest: &str) -> Result<bool, Refused> {
        let url = format!("https://{}/v2/{}/blobs/{digest}", self.api, self.name);
        let response = self.send(self.client.http().head(&url)).await?;
        match response.status() {
            StatusCode::OK => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Err(refused(&url, &response)),
        }
    }

    /// The blob under `digest`, `size` bytes of `body`, uploaded whole
    /// in one request: an upload opened, and the bytes put to where
    /// the registry said, naming the digest they must be.
    pub async fn put_blob(&self, digest: &str, size: u64, body: Body) -> Result<(), Refused> {
        let open = format!("https://{}/v2/{}/blobs/uploads/", self.api, self.name);
        let response = self.send(self.client.http().post(&open).header(header::CONTENT_LENGTH, 0)).await?;
        if response.status() != StatusCode::ACCEPTED {
            return Err(refused(&open, &response));
        }
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| Refused::Other(format!("`{open}` opened an upload and named no place for it")))?;
        // The place may be relative, and may carry a query of its own.
        let place = if location.starts_with("http://") || location.starts_with("https://") {
            location.to_string()
        } else {
            format!("https://{}{location}", self.api)
        };
        let joiner = if place.contains('?') { '&' } else { '?' };
        let url = format!("{place}{joiner}digest={digest}");
        let request = self
            .client
            .http()
            .put(&url)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, size)
            .body(body);
        let response = self.send(request).await?;
        match response.status() {
            StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            _ => Err(refused(&url, &response)),
        }
    }

    /// The manifest `body`, served as `media_type`, put under
    /// `reference`: a tag, or the manifest's own digest.
    pub async fn put_manifest(&self, reference: &str, media_type: &str, body: Vec<u8>) -> Result<(), Refused> {
        let url = format!("https://{}/v2/{}/manifests/{reference}", self.api, self.name);
        let request = self.client.http().put(&url).header(header::CONTENT_TYPE, media_type).body(body);
        let response = self.send(request).await?;
        match response.status() {
            StatusCode::CREATED | StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
            _ => Err(refused(&url, &response)),
        }
    }

    /// The request sent, presenting what the registry takes.
    async fn send(&self, request: RequestBuilder) -> Result<Response, Refused> {
        let request = match &self.auth {
            Auth::None => request,
            Auth::Basic(username, password) => request.basic_auth(username, Some(password)),
            Auth::Bearer(token) => request.bearer_auth(token),
        };
        request.send().await.map_err(other)
    }
}

/// A response's `WWW-Authenticate`, or nothing.
fn challenge(response: &Response) -> String {
    response
        .headers()
        .get(header::WWW_AUTHENTICATE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

/// A write's answer that is not the one wanted, by rule 3 of
/// [`Writer`].
fn refused(url: &str, response: &Response) -> Refused {
    match response.status() {
        StatusCode::UNAUTHORIZED if challenge(response).contains("insufficient_scope") => Refused::Denied,
        StatusCode::UNAUTHORIZED => Refused::Unauthorized,
        StatusCode::FORBIDDEN => Refused::Denied,
        status => Refused::Other(format!("`{url}` answered {status}")),
    }
}

/// A transport's failure, in its own words.
fn other(error: reqwest::Error) -> Refused {
    Refused::Other(error.to_string())
}
