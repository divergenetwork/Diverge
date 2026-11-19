//! The releases, asked.

use std::path::Path;

use serde::Deserialize;
use tokio::io::AsyncWriteExt as _;

use super::Error;

/// The variable that names the releases' address in place of
/// [`ADDRESS`].
pub const DIVERGE_RELEASES_URL: &str = "DIVERGE_RELEASES_URL";

/// Where the releases are listed and downloaded from.
pub const ADDRESS: &str = "https://releases.diverge.network";

/// The releases' address: [`DIVERGE_RELEASES_URL`] where it is set
/// and not empty, else [`ADDRESS`], with no trailing slash.
pub fn address() -> String {
    let address = std::env::var(DIVERGE_RELEASES_URL).ok().filter(|address| !address.is_empty());
    address.unwrap_or_else(|| ADDRESS.to_string()).trim_end_matches('/').to_string()
}

/// The answer of `GET /`: every version, newest first.
#[derive(Debug, Clone, Deserialize)]
struct Versions {
    versions: Vec<Listed>,
}

/// One version of [`Versions`].
#[derive(Debug, Clone, Deserialize)]
struct Listed {
    version: String,
}

/// The answer of `GET /<version>`: the release's platforms.
#[derive(Debug, Clone, Deserialize)]
struct Release {
    platforms: Vec<Platform>,
}

/// One platform of a [`Release`].
#[derive(Debug, Clone, Deserialize)]
struct Platform {
    os: String,
    arch: String,
    file: String,
}

/// The HTTP client every ask goes out on. It follows redirects, which
/// is what a download is.
pub fn client() -> Result<reqwest::Client, Error> {
    reqwest::Client::builder()
        .user_agent(concat!("diverge/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|source| Error::Http { url: address(), source })
}

/// `GET url`, asked for JSON, answered with success.
async fn get(client: &reqwest::Client, url: &str) -> Result<reqwest::Response, Error> {
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|source| Error::Http { url: url.to_string(), source })?;
    if !response.status().is_success() {
        return Err(Error::Status { url: url.to_string(), status: response.status().as_u16() });
    }
    Ok(response)
}

/// The version an update with none asked for installs, or `None`
/// where there is nothing to do.
///
/// The rules, over the releases' list, newest first:
///
/// 1. `current` is the newest: `None`.
/// 2. `current` is in the list and not the newest: the version next
///    after it — one version up, whatever is newer still.
/// 3. `current` is not in the list: the newest.
pub async fn next(client: &reqwest::Client, address: &str, current: &str) -> Result<Option<String>, Error> {
    let url = format!("{address}/");
    let listed: Versions = get(client, &url).await?.json().await.map_err(|source| Error::Http { url, source })?;
    let versions: Vec<String> = listed.versions.into_iter().map(|listed| listed.version).collect();
    Ok(match versions.iter().position(|version| version == current) {
        Some(0) => None,
        Some(index) => Some(versions[index - 1].clone()),
        None => versions.into_iter().next(),
    })
}

/// The archive's file name for `version` on `os` and `arch`, as the
/// release lists it. A version there is none of is
/// [`Error::NoVersion`], and a platform it was not built for
/// [`Error::NoPlatform`].
pub async fn file(client: &reqwest::Client, address: &str, version: &str, os: &str, arch: &str) -> Result<String, Error> {
    let url = format!("{address}/{version}");
    let release: Release = match get(client, &url).await {
        Ok(response) => response.json().await.map_err(|source| Error::Http { url, source })?,
        Err(Error::Status { status: 404, .. }) => return Err(Error::NoVersion(version.to_string())),
        Err(error) => return Err(error),
    };
    release
        .platforms
        .into_iter()
        .find(|platform| platform.os == os && platform.arch == arch)
        .map(|platform| platform.file)
        .ok_or_else(|| Error::NoPlatform { version: version.to_string(), os: os.to_string(), arch: arch.to_string() })
}

/// The archive of `version` for `os` and `arch`, downloaded to `to`,
/// piece by piece as it comes. The address asked is the releases'
/// own, which redirects to where the file is.
pub async fn download(client: &reqwest::Client, address: &str, version: &str, os: &str, arch: &str, to: &Path) -> Result<(), Error> {
    let url = format!("{address}/{version}/{os}/{arch}");
    let mut response = client.get(&url).send().await.map_err(|source| Error::Http { url: url.clone(), source })?;
    if !response.status().is_success() {
        return Err(Error::Status { url, status: response.status().as_u16() });
    }
    let failed = |source| Error::Io { path: to.to_path_buf(), source };
    let mut file = tokio::fs::File::create(to).await.map_err(failed)?;
    while let Some(piece) = response.chunk().await.map_err(|source| Error::Http { url: url.clone(), source })? {
        file.write_all(&piece).await.map_err(failed)?;
    }
    file.flush().await.map_err(failed)
}

/// The skills of `version`, downloaded to `to`, piece by piece as they
/// come: one zip, the same for every platform, asked at the
/// releases' own address, which redirects to where the file is. A
/// version released with none — every one before skills were — is
/// [`Error::NoSkills`].
pub async fn download_skills(client: &reqwest::Client, address: &str, version: &str, to: &Path) -> Result<(), Error> {
    let url = format!("{address}/{version}/skills");
    let mut response = client.get(&url).send().await.map_err(|source| Error::Http { url: url.clone(), source })?;
    if response.status().as_u16() == 404 {
        return Err(Error::NoSkills(version.to_string()));
    }
    if !response.status().is_success() {
        return Err(Error::Status { url, status: response.status().as_u16() });
    }
    let failed = |source| Error::Io { path: to.to_path_buf(), source };
    let mut file = tokio::fs::File::create(to).await.map_err(failed)?;
    while let Some(piece) = response.chunk().await.map_err(|source| Error::Http { url: url.clone(), source })? {
        file.write_all(&piece).await.map_err(failed)?;
    }
    file.flush().await.map_err(failed)
}
