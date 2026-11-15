//! The podman the provider uses.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Registry;

/// Everything the provider hands podman, whatever it uses podman for
/// — running containers, building an image, reading a file out of
/// one: where it pulls from, where it keeps its data, and how much
/// the image cache may hold.
///
/// Any field may be left out, and is then its default; the section
/// absent is every field its default. A provider that uses no podman
/// — one with neither a `containers` section nor an `images` section
/// — reads none of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Podman {
    /// The registries the provider will look in when an image's
    /// references name them, each with the credential the provider
    /// presents to it, if any: for an image it does not hold, every
    /// referenced registry listed here is asked at once, beside the
    /// caller, and the first to have the image is pulled from; a
    /// reference naming a registry not listed here is ignored. The
    /// provider's own registry, which serves what a caller holds, is
    /// not listed here and needs no entry.
    pub registries: Vec<Registry>,
    /// The directory podman's data is kept under: the image cache,
    /// what a container writes over its image, and what holds a
    /// volume's changes apart when its mode is `ephemeral`. On Linux it
    /// is podman's storage root, the `graphroot`: every podman
    /// invocation the provider makes is given it as `--root`, so it
    /// is the whole store the provider sees — an image loaded into
    /// it by hand is seen there, and one in podman's default store is
    /// not; a changed path is a fresh, empty store, and the old one
    /// is left as it was. On macOS and Windows it is where the podman machine
    /// lives, description and disk: the provider makes the machine
    /// under this path, a changed path is a fresh machine under the
    /// new one, and the machine under the old path is left as it
    /// was, for the operator to stop or remove when they choose; the
    /// data lies inside the machine's disk. A path that is not
    /// absolute is resolved relative to the directory that contains
    /// `provider.yaml` itself, never to the working directory; an
    /// absolute path stands as written.
    ///
    /// A container's `disk` is enforced as podman's storage size
    /// option on the container, which the overlay driver keeps with
    /// the kernel's project quotas. So the filesystem under this path
    /// has them: XFS mounted with `pquota`, or ext4 with project
    /// quota enabled. On any other filesystem podman refuses the
    /// option, and every run fails with its refusal; the provider
    /// does not probe for it.
    pub storage_path: PathBuf,
    /// The most the image cache may hold, in BYTES: the layers of
    /// every image in the store, kept for the next run of it. The
    /// provider removes images no running container uses to stay
    /// under it, least recently run first, except an image whose
    /// config carries the label `diverge.network/keep`, which is
    /// never removed and always counted; an image larger than it
    /// alone cannot be pulled.
    pub image_cache_disk: u64,
}

/// What a `podman` section that names nothing gives: the three
/// registries a reference may name, `docker.io` first, each pulled
/// from anonymously; podman's data under `provider/podman_data` beside
/// `provider.yaml`; and a 32 GiB image cache.
impl Default for Podman {
    fn default() -> Self {
        Podman {
            registries: ["docker.io", "ghcr.io", "quay.io"]
                .iter()
                .map(|host| Registry {
                    host: host.to_string(),
                    credential: None,
                })
                .collect(),
            storage_path: PathBuf::from("provider/podman_data"),
            // 32 GiB.
            image_cache_disk: 32 * 1024 * 1024 * 1024,
        }
    }
}
