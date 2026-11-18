//! A manifest read: an index, or an image manifest, and the platform
//! the provider takes from an index.

use serde::Serialize;
use serde_json::Value;

use diverge_sdk::shared::platform;

use crate::host::image_registry::Digest;

/// The OCI image manifest's media type: what a manifest that names
/// none is served as.
pub const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
/// The OCI image index's media type.
pub const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// Docker's image manifest, schema 2.
pub const DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
/// Docker's manifest list, schema 2: its index.
pub const DOCKER_INDEX: &str = "application/vnd.docker.distribution.manifest.list.v2+json";

/// Every manifest media type the provider accepts, as an `Accept`
/// header lists them.
pub const ACCEPT: &str = "application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json";

/// One descriptor: a digest, parsed, and the size it declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descriptor {
    pub digest: Digest,
    pub size: u64,
}

/// One entry of an index: its descriptor, and the platform it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub descriptor: Descriptor,
    pub os: Option<String>,
    pub architecture: Option<String>,
    pub variant: Option<String>,
}

impl Entry {
    /// The platform the entry is for, as the one string a platform is
    /// named by: see [`platform::of`]. An entry that states no `os` or
    /// no `architecture` is for `unknown` in its place, which is what
    /// an index says of an entry that is no platform's — an
    /// attestation — and names it all the same.
    pub fn platform(&self) -> String {
        platform::of(self.os.as_deref().unwrap_or("unknown"), self.architecture.as_deref().unwrap_or("unknown"), self.variant.as_deref())
    }
}

/// The platforms an index names: each entry's, each once, in the
/// index's order.
pub fn named(entries: &[Entry]) -> Vec<String> {
    let mut platforms: Vec<String> = Vec::new();
    for entry in entries {
        let platform = entry.platform();
        if !platforms.contains(&platform) {
            platforms.push(platform);
        }
    }
    platforms
}

/// The one platform an image that is not an index is for, as its
/// config states it: `os`, `architecture` and, where there is one,
/// `variant`. `None` for a config that is not JSON or states neither.
pub fn config_platform(config: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(config).ok()?;
    let field = |name: &str| value.get(name).and_then(Value::as_str);
    Some(platform::of(field("os")?, field("architecture")?, field("variant")))
}

/// One entry of an index a merge writes: a platform's image manifest,
/// and the platform, as its parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The image manifest's digest.
    pub digest: Digest,
    /// Its size in bytes.
    pub size: u64,
    /// The media type it is served as.
    pub media_type: String,
    /// The platform: `os/architecture`, or with a `/variant`.
    pub platform: String,
}

/// The index a merge writes over `members`, as bytes. The rules:
///
/// 1. **An OCI image index**, `schemaVersion` 2, with its `mediaType`.
/// 2. **One form.** The entries are ordered by platform, then by
///    digest, and every object's members are written in one fixed
///    order, with no whitespace: the same members are the same bytes,
///    and so the same digest, whoever writes them and in whatever
///    order they were given.
/// 3. **A platform is its parts.** `os`, `architecture`, and `variant`
///    where the platform string has a third part.
pub fn index(members: &[Member]) -> Vec<u8> {
    #[derive(Serialize)]
    struct Platform<'a> {
        architecture: &'a str,
        os: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        variant: Option<&'a str>,
    }
    #[derive(Serialize)]
    struct Manifest<'a> {
        #[serde(rename = "mediaType")]
        media_type: &'a str,
        digest: String,
        size: u64,
        platform: Platform<'a>,
    }
    #[derive(Serialize)]
    struct Index<'a> {
        #[serde(rename = "schemaVersion")]
        schema_version: u8,
        #[serde(rename = "mediaType")]
        media_type: &'a str,
        manifests: Vec<Manifest<'a>>,
    }
    let mut ordered: Vec<&Member> = members.iter().collect();
    ordered.sort_by(|a, b| a.platform.cmp(&b.platform).then_with(|| a.digest.to_string().cmp(&b.digest.to_string())));
    let manifests = ordered
        .into_iter()
        .map(|member| {
            let mut parts = member.platform.splitn(3, '/');
            let os = parts.next().unwrap_or("unknown");
            let architecture = parts.next().unwrap_or("unknown");
            Manifest {
                media_type: &member.media_type,
                digest: member.digest.to_string(),
                size: member.size,
                platform: Platform { architecture, os, variant: parts.next() },
            }
        })
        .collect();
    // A struct of strings and numbers always serializes.
    serde_json::to_vec(&Index { schema_version: 2, media_type: OCI_INDEX, manifests }).unwrap_or_default()
}

/// A manifest, read.
///
/// The rules:
///
/// 1. A manifest is JSON; anything else is not one.
/// 2. Its kind is its `mediaType` when it names one: one of the two
///    index types is an [`Index`](Self::Index), one of the two image
///    manifest types an [`Image`](Self::Image), and any other is not a
///    manifest the provider takes. With no `mediaType`, which the OCI
///    specification allows, a `manifests` array is an index and a
///    `config` an image manifest.
/// 3. Every descriptor has a parsed digest and a `size`; one without
///    either makes the whole manifest not one.
/// 4. An image manifest's `layers` are kept in the order listed,
///    duplicates and all: the order and the count are the proof's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// An index: the manifests it lists, one per platform.
    Index(Vec<Entry>),
    /// An image manifest: its config, then its layers.
    Image {
        config: Descriptor,
        layers: Vec<Descriptor>,
    },
}

impl Parsed {
    /// The manifest in `body`, or `None` for bytes that break a rule.
    pub fn parse(body: &[u8]) -> Option<Parsed> {
        let value: Value = serde_json::from_slice(body).ok()?;
        let index = match value.get("mediaType").and_then(Value::as_str) {
            Some(OCI_INDEX | DOCKER_INDEX) => true,
            Some(OCI_MANIFEST | DOCKER_MANIFEST) => false,
            Some(_) => return None,
            None if value.get("manifests").is_some() => true,
            None if value.get("config").is_some() => false,
            None => return None,
        };
        if index {
            let entries = value.get("manifests")?.as_array()?;
            let mut parsed = Vec::with_capacity(entries.len());
            for entry in entries {
                let platform = entry.get("platform");
                let field = |name: &str| platform.and_then(|platform| platform.get(name)).and_then(Value::as_str).map(str::to_string);
                parsed.push(Entry {
                    descriptor: descriptor(entry)?,
                    os: field("os"),
                    architecture: field("architecture"),
                    variant: field("variant"),
                });
            }
            Some(Parsed::Index(parsed))
        } else {
            let config = descriptor(value.get("config")?)?;
            let mut layers = Vec::new();
            for layer in value.get("layers")?.as_array()? {
                layers.push(descriptor(layer)?);
            }
            Some(Parsed::Image { config, layers })
        }
    }
}

/// One descriptor's digest and size, or `None`.
fn descriptor(value: &Value) -> Option<Descriptor> {
    Some(Descriptor {
        digest: Digest::parse(value.get("digest")?.as_str()?)?,
        size: value.get("size")?.as_u64()?,
    })
}

/// The media type a manifest is served as: the one it names, or — for
/// one that names none — the OCI index's or image manifest's, by its
/// kind.
pub fn media_type(body: &[u8]) -> String {
    let value: Option<Value> = serde_json::from_slice(body).ok();
    match value.as_ref().and_then(|value| value.get("mediaType")).and_then(Value::as_str) {
        Some(named) => named.to_string(),
        None => match Parsed::parse(body) {
            Some(Parsed::Index(_)) => OCI_INDEX.to_string(),
            _ => OCI_MANIFEST.to_string(),
        },
    }
}

/// The platform the provider runs: `linux`, since every container is
/// a Linux one, on this host's architecture as the OCI specification
/// names it — the machine podman runs in on macOS and Windows is of
/// the host's architecture too.
pub fn architecture() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "x86" => "386",
        "arm" => "arm",
        "powerpc64" => "ppc64le",
        other => other,
    }
}

/// The platform this provider runs and builds for, as a platform
/// string: `linux/<architecture>`.
pub fn host() -> String {
    platform::of("linux", architecture(), None)
}

/// The entry of an index the provider takes for a RUN: the first
/// whose `os` is `linux` and whose `architecture` is
/// [`architecture`]'s, which is the one podman pulls on the same
/// host. `None` for an index with no such entry: the image does not
/// run here.
pub fn select(entries: &[Entry]) -> Option<&Entry> {
    entries
        .iter()
        .find(|entry| entry.os.as_deref() == Some("linux") && entry.architecture.as_deref() == Some(architecture()))
}
