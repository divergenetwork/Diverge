//! What `diverge provider` takes.

use clap::Subcommand;

use super::{Action, Identity, Lifecycle, Port};

/// The provider on this host: started, stopped, its document in the
/// root worked on whole, or one setting of that document — its port,
/// its identity, whom it accepts, whom it dials, podman's settings,
/// whether it runs containers, the volumes it provides — read or
/// written by itself.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Server {
    /// Start or stop the server: see [`lifecycle`](crate::lifecycle).
    #[command(flatten)]
    Lifecycle(Lifecycle),
    /// Read, rewrite or edit the provider's document in the root,
    /// `provider.yaml`.
    #[command(subcommand)]
    Config(Action),
    /// Read or write the TCP port the provider accepts the protocol
    /// on, `port` in `provider.yaml`.
    #[command(subcommand)]
    Port(Port),
    /// Read or write who the provider is, `identity` in
    /// `provider.yaml`.
    #[command(subcommand)]
    Identity(Identity),
    /// List the identities the provider accepts, add one, or delete
    /// one, `incoming` in `provider.yaml`.
    #[command(subcommand)]
    Incoming(Incoming),
    /// List the peers the provider dials, add one, or delete one,
    /// `outgoing` in `provider.yaml`.
    #[command(subcommand)]
    Outgoing(Outgoing),
    /// Read or write one setting of the podman the provider uses,
    /// `podman` in `provider.yaml`.
    #[command(subcommand)]
    Podman(Podman),
    /// Turn the running of containers on or off, or read or write one
    /// setting of it, `containers` in `provider.yaml`.
    #[command(subcommand)]
    Containers(Containers),
    /// Turn the `volumes` section on or off, or list, add or delete
    /// the volumes the provider provides, `volumes` in
    /// `provider.yaml`.
    #[command(subcommand)]
    Volumes(Volumes),
}

/// What is done to the provider's `volumes` section: see
/// [`provider`](crate::cli::provider). The section holds two lists,
/// the stores a volume may be created in and the fixed volumes the
/// provider provides; the provider keeps volumes when either holds
/// something, and none when both are empty, the section there or not.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Volumes {
    /// Print whether the section is there: `true` or `false`. A
    /// section that is there with nothing in it still keeps no
    /// volume.
    Enabled,
    /// Make the section, with nothing in it. Nothing changes where it
    /// is there already. Refused while the provider is running on the
    /// root: stop it first.
    Enable,
    /// Remove the section, and both its lists with it. Refused while
    /// the provider is running on the root: stop it first.
    Disable,
    /// List, add or delete the volumes the provider provides as they
    /// are — each a directory on the host, listed to clients as
    /// `provided`, which no client edits or deletes —
    /// `volumes.fixed` in `provider.yaml`.
    #[command(subcommand)]
    Fixed(Fixed),
    /// List, add, edit or delete the places a volume may be created
    /// in, `volumes.stores` in `provider.yaml`. Each is a folder that
    /// holds every volume created in it: editing a store's path moves
    /// the folder, and deleting a store deletes it.
    #[command(subcommand)]
    Stores(Stores),
}

/// What is done to the provider's `volumes.stores`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Stores {
    /// Print the stores, one line each, in the document's order,
    /// which is the order of preference:
    /// `{"index":0,"path":"…","capacity":…}`. Nothing is printed
    /// where there is none, or where the section is absent. The index
    /// is what `edit` and `delete` take.
    Get,
    /// Print how much of each store is taken, one line each:
    /// `{"index":0,"capacity":…,"used":…,"free":…}`, in BYTES. `used`
    /// is what the volumes in the store reserve together, which is
    /// what counts against `capacity`, and is the least an `edit` may
    /// set the capacity to. Nothing is printed where there is no
    /// store.
    Capacity,
    /// Add a store, after the last. Refused where the section is
    /// absent — enable it first — where the path is a store's
    /// already, and while the provider is running on the root. The
    /// folder is made by the provider at its start.
    Add {
        /// The folder: absolute, or relative to the root.
        #[arg(long)]
        path: std::path::PathBuf,
        /// The most the volumes in it may reserve together, in BYTES.
        #[arg(long)]
        capacity: u64,
    },
    /// Change a store's path, its capacity, or both. A new path MOVES
    /// the store: the folder and every volume in it is carried there
    /// before the path is written — renamed where the two paths are
    /// on one filesystem, copied, checked and the old copy deleted
    /// where they are not — and if it cannot be, nothing is moved and
    /// nothing is written. Refused while the provider is running on
    /// the root.
    Edit {
        /// The store's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
        /// The new folder: absolute, or relative to the root. It must
        /// not be there, or be empty.
        #[arg(long, required_unless_present = "capacity")]
        path: Option<std::path::PathBuf>,
        /// The new capacity, in BYTES. Refused where it is less than
        /// what the volumes in the store reserve now: see `capacity`.
        #[arg(long)]
        capacity: Option<u64>,
    },
    /// Delete a store: its folder and EVERY VOLUME IN IT, then its
    /// entry. Not undone. Refused while the provider is running on
    /// the root. Every store after it moves one index down.
    Delete {
        /// The store's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// What is done to the provider's `volumes.fixed`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Fixed {
    /// Print the fixed volumes, one line each, in the document's
    /// order: `{"index":0,"name":"…","path":"…","bytes":…,"mode":"…"}`,
    /// with `authorize_hook` where it has one. Nothing is printed
    /// where there is none, or where the section is absent. The index
    /// is what `delete` takes.
    Get,
    /// Add a fixed volume, after the last. Refused where the section
    /// is absent — enable it first — where the name is listed
    /// already, and while the provider is running on the root.
    Add {
        /// The name clients list and mount it under.
        #[arg(long)]
        name: String,
        /// The directory on the host: absolute, or relative to the
        /// root. Written as given.
        #[arg(long)]
        path: std::path::PathBuf,
        /// The size it is listed with, in BYTES: declared, not
        /// measured.
        #[arg(long)]
        bytes: u64,
        /// The mode it is in: `persistent`, `ephemeral` or
        /// `read_only`.
        #[arg(long, value_parser = mode)]
        mode: diverge_sdk::provider::endpoints::volumes::Mode,
        /// The hook that says which identities are listed it: a folder
        /// under the provider's `hooks/`. Absent, every identity is.
        #[arg(long)]
        authorize_hook: Option<String>,
    },
    /// Delete the fixed volume at an index: the entry, never the
    /// directory. Refused where the section is absent and while the
    /// provider is running on the root. Every one after it moves one
    /// index down.
    Delete {
        /// The volume's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// A volume mode from its name on the wire, snake case.
fn mode(name: &str) -> Result<diverge_sdk::provider::endpoints::volumes::Mode, String> {
    serde_json::from_value(serde_json::Value::String(name.to_string()))
        .map_err(|_| "one of `persistent`, `ephemeral`, `read_only`".to_string())
}

/// What is done to the provider's `containers` section: see
/// [`provider`](crate::cli::provider). The section there is the
/// provider running containers, and the section absent is it running
/// none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Containers {
    /// Print whether the provider runs containers: `true` or `false`.
    Enabled,
    /// Have the provider run containers: the section made, with its
    /// defaults. Nothing changes where it is there already. Refused
    /// while the provider is running on the root: stop it first.
    Enable,
    /// Have the provider run no containers: the section removed, and
    /// its settings with it. Refused while the provider is running on
    /// the root: stop it first.
    Disable,
    /// Read or write the most every running container's disk may
    /// take together, `containers.container_overlay_disk`.
    #[command(name = "container_overlay_disk", subcommand)]
    ContainerOverlayDisk(Bytes),
    /// Read or write the most every running container's memory may
    /// take together, `containers.memory`; on macOS, also the memory
    /// the podman machine is given.
    #[command(subcommand)]
    Memory(Bytes),
}

/// What is done to a setting that is a number of bytes, in a section
/// that may be absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Bytes {
    /// Print the number of bytes, or `null` where the section is
    /// absent: the feature is off, and no number is in effect.
    Get,
    /// Write the number of bytes. Refused where the section is absent
    /// — enable it first — and while the provider is running on the
    /// root.
    Set {
        /// The number of BYTES: a plain integer, `34359738368` for
        /// 32 GiB.
        #[arg(value_name = "BYTES")]
        value: u64,
    },
}

/// The settings of the provider's `podman` section, each a command
/// beneath `diverge provider podman`, named as the section names it.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Podman {
    /// List the registries the provider pulls from, add one, or
    /// delete one, `podman.registries` in `provider.yaml`.
    #[command(subcommand)]
    Registries(Registries),
    /// Read where podman keeps its data, or move the data and write
    /// the new place, `podman.storage_path` in `provider.yaml`.
    #[command(name = "storage_path", subcommand)]
    StoragePath(StoragePath),
    /// Read or write the most podman's image cache may hold,
    /// `podman.image_cache_disk` in `provider.yaml`.
    #[command(name = "image_cache_disk", subcommand)]
    ImageCacheDisk(ImageCacheDisk),
}

/// What is done to the provider's `podman.image_cache_disk`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum ImageCacheDisk {
    /// Print the number of bytes, as the provider reads it: the
    /// default, 32 GiB, where the document has none.
    Get,
    /// Write the number of bytes. Refused while the provider is
    /// running on the root: stop it first.
    Set {
        /// The most the image cache may hold, in BYTES: a plain
        /// integer, `34359738368` for 32 GiB.
        #[arg(value_name = "BYTES")]
        value: u64,
    },
}

/// What is done to the provider's `podman.storage_path`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum StoragePath {
    /// Print the path, resolved: absolute, a relative one taken from
    /// the root, as the provider takes it.
    Get,
    /// Move podman's data to the path and write it. Refused while the
    /// provider is running on the root: stop it first. The data is
    /// moved whole — on macOS and Windows that is the podman machine,
    /// stopped first if it runs — by renames where the two paths are
    /// on one filesystem, and by a checked copy where they are not;
    /// if it cannot be, nothing is moved and the path is not written.
    Set {
        /// The path: absolute, or relative to the root.
        #[arg(value_name = "PATH")]
        path: std::path::PathBuf,
        /// Write the path and move nothing: podman's data is left at
        /// the old path, and the provider starts with none at the
        /// new. Refused on Windows while the old path's machine is
        /// there, beside which the provider would not start.
        #[arg(long)]
        no_transfer: bool,
    },
    /// Delete everything podman keeps at the path, so that the
    /// provider makes it all again at its next start: an empty store,
    /// or on macOS and Windows a machine made new. Refused while the
    /// provider is running on the root: stop it first. Every image
    /// kept there is gone and is not got back; the path itself stays
    /// as it is written.
    Delete,
}

/// What is done to the provider's `podman.registries`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Registries {
    /// Print the registries, one line each, in the document's order:
    /// `{"index":0,"host":"docker.io"}`. The index is what `delete`
    /// takes.
    Get,
    /// Add a registry, after the last. Refused while the provider is
    /// running on the root: stop it first. Refused where the host is
    /// listed already.
    Add {
        /// The registry's host, `docker.io`, `ghcr.io`,
        /// `registry.example.com:5000`: what an image reference names
        /// before its first `/`, with no scheme and no path.
        #[arg(value_name = "HOST")]
        host: String,
    },
    /// Delete the registry at an index. Refused while the provider is
    /// running on the root: stop it first. Every registry after it
    /// moves one index down.
    Delete {
        /// The registry's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// What is done to the provider's `incoming`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Incoming {
    /// Print the identities the provider accepts, one line each, in
    /// the document's order: `{"index":0,"issuer":"…","subject":"…"}`.
    /// The index is what `delete` takes.
    Get,
    /// Add an identity, after the last. Refused while the provider is
    /// running on the root: stop it first. Refused where the identity
    /// is listed already.
    Add {
        /// The identity's issuer, exactly as its discovery document
        /// names it.
        #[arg(long)]
        issuer: String,
        /// The identity's subject: the `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
    /// Delete the identity at an index. Refused while the provider is
    /// running on the root: stop it first. Every identity after it
    /// moves one index down.
    Delete {
        /// The identity's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}

/// What is done to the provider's `outgoing`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Outgoing {
    /// Print the peers the provider dials, one line each, in the
    /// document's order:
    /// `{"index":0,"address":"…","issuer":"…","subject":"…"}`. The
    /// index is what `delete` takes.
    Get,
    /// Add a peer, after the last. Refused while the provider is
    /// running on the root: stop it first. Refused where the same
    /// identity at the same address is listed already.
    Add {
        /// Where the peer listens: `host:port`, dialled as
        /// `ws://<address>/`, or a URL with its own scheme.
        #[arg(long)]
        address: String,
        /// The issuer of the identity that must answer there, exactly
        /// as its discovery document names it.
        #[arg(long)]
        issuer: String,
        /// The subject of the identity that must answer there: the
        /// `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
    /// Delete the peer at an index. Refused while the provider is
    /// running on the root: stop it first. Every peer after it moves
    /// one index down.
    Delete {
        /// The peer's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
}
