//! What `diverge provider` takes.

use clap::{Args, Subcommand};

use super::{Identity, Lifecycle, Port};

/// The provider on this host: started, stopped, or one setting of it
/// — its port,
/// its identity, whom it accepts, whom it dials, podman's settings,
/// whether it runs containers, its volumes, its image Store, the
/// messages it relays — read or written by itself. The settings are
/// not worked on whole: there is no `provider config`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Server {
    /// Start or stop the server: see [`lifecycle`](crate::lifecycle).
    #[command(flatten)]
    Lifecycle(Lifecycle),
    /// Read or write the TCP port the provider accepts the protocol
    /// on.
    #[command(subcommand)]
    Port(Port),
    /// Read or write who the provider is.
    #[command(subcommand)]
    Identity(Identity),
    /// List the identities the provider accepts, add one, or delete
    /// one.
    #[command(subcommand)]
    Incoming(Incoming),
    /// List the peers the provider dials, add one, or delete one.
    #[command(subcommand)]
    Outgoing(Outgoing),
    /// Read or write one setting of the podman the provider uses.
    #[command(subcommand)]
    Podman(Podman),
    /// Turn the running of containers on or off, or read or write one
    /// setting of it.
    #[command(subcommand)]
    Containers(Containers),
    /// Turn the `volumes` section on or off, or list, add or delete
    /// the volumes the provider provides.
    #[command(subcommand)]
    Volumes(Volumes),
    /// Turn the provider's image Store on or off, or read or write
    /// one setting of it.
    #[command(subcommand)]
    Images(Images),
    /// Turn the relaying of messages on or off, or read or write the
    /// rate it is relayed at.
    #[command(subcommand)]
    Messages(Messages),
}

/// What is done to the provider's `messages` section: see
/// [`provider`](crate::cli::provider). The section there is the
/// provider relaying direct messages between the identities
/// connected to it, at most `messages` sends per sending identity in
/// every `seconds`; the section absent is it relaying none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Messages {
    /// Print whether the provider relays messages: `true` or `false`.
    Enabled,
    /// Have the provider relay messages: the section made, with its
    /// defaults, 30 messages in every 60 seconds. Nothing changes
    /// where it is there already. Refused while the provider is
    /// running on the root: stop it first.
    Enable,
    /// Have the provider relay no messages: the section removed, and
    /// its rate with it. Refused while the provider is running on the
    /// root: stop it first.
    Disable,
    /// Read or write the most sends one identity makes in one window,
    /// `messages.messages`.
    #[command(subcommand)]
    Messages(Rate),
    /// Read or write how long one window is, in seconds,
    /// `messages.seconds`.
    #[command(subcommand)]
    Seconds(Rate),
}

/// What is done to one of the two numbers of the rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Rate {
    /// Print the number, or `null` where the section is absent: no
    /// message is relayed, and no rate is in effect.
    Get,
    /// Write the number. Refused where the section is absent —
    /// enable it first — and while the provider is running on the
    /// root.
    Set {
        /// The number: `1` or more.
        #[arg(value_name = "NUMBER")]
        value: std::num::NonZeroU32,
    },
}

/// What is done to the provider's `images` section: see
/// [`provider`](crate::cli::provider). The section there is the
/// provider keeping a Store — the images clients build and push on
/// it — and the section absent is it keeping none.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Images {
    /// Print whether the provider keeps a Store: `true` or `false`.
    Enabled,
    /// Have the provider keep a Store: the section made, with its
    /// defaults — a path of its own under the root, which
    /// `images path get` prints, and a capacity of 32 GiB. Nothing changes where it is there already.
    /// Refused while the provider is running on the root: stop it
    /// first.
    Enable,
    /// Have the provider keep no Store: the section removed, and its
    /// settings with it. The folder and every image in it are left as
    /// they are. Refused while the provider is running on the root.
    Disable,
    /// Read where the Store is kept, move it, or delete it,
    /// `images.path`.
    #[command(subcommand)]
    Path(ImagesPath),
    /// Read or write the most the Store may hold, `images.capacity`.
    #[command(subcommand)]
    Capacity(ImagesCapacity),
    /// Print how much of the Store is taken:
    /// `{"capacity":…,"used":…,"cached":…,"free":…}`, in BYTES, or
    /// `null` where the section is absent. `used` is what the images
    /// clients own take, which is the least `capacity set` may set;
    /// `cached` is what is kept besides, which the provider removes
    /// whenever room is needed.
    #[command(name = "capacity_used")]
    CapacityUsed,
}

/// What is done to the provider's `images.path`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ImagesPath {
    /// Print the path, resolved: absolute, a relative one taken from
    /// the root. `null` where the section is absent.
    Get,
    /// Move the Store to the path and write it. Refused where the
    /// section is absent, and while the provider is running on the
    /// root. The Store is moved whole — renamed where the two paths
    /// are on one filesystem, copied, checked and the old copy
    /// deleted where they are not — and if it cannot be, nothing is
    /// moved and the path is not written.
    Set {
        /// The path: absolute, or relative to the root. It must not
        /// be there, or be empty.
        #[arg(value_name = "PATH")]
        path: std::path::PathBuf,
        /// Write the path and move nothing: the Store is left at the
        /// old path, and the provider starts with an empty one at the
        /// new.
        #[arg(long)]
        no_transfer: bool,
    },
    /// Delete the Store's folder and EVERY IMAGE IN IT, so that the
    /// provider starts with an empty Store. Not undone. The path and
    /// the section stay as they are written. Refused while the
    /// provider is running on the root.
    Delete,
}

/// What is done to the provider's `images.capacity`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum ImagesCapacity {
    /// Print the number of bytes, or `null` where the section is
    /// absent.
    Get,
    /// Write the number of bytes. Refused where the section is
    /// absent, where it is less than what the images clients own
    /// take now — see `capacity_used` — and while the provider is
    /// running on the root.
    Set {
        /// The most the Store may hold, in BYTES.
        #[arg(value_name = "BYTES")]
        value: u64,
    },
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
    /// `provided`, which no client edits or deletes — or, after a
    /// volume's index, read or write who that volume is listed to.
    Fixed(Fixed),
    /// List, add, edit or delete the places a volume may be created
    /// in. Each is a folder that
    /// holds every volume created in it: editing a store's path moves
    /// the folder, and deleting a store deletes it.
    #[command(subcommand)]
    Stores(Stores),
    /// Read, set or delete where a FUSE serve of an ephemeral volume
    /// keeps what is written through it, and how much,
    /// `volumes.ephemeral_fuse`. With it set the provider serves
    /// ephemeral volumes over FUSE; deleted, it serves none. It is set
    /// from the moment the `volumes` section is enabled, to
    /// `provider/ephemeral_fuse` under the root and 32 GiB.
    #[command(name = "ephemeral_fuse", subcommand)]
    EphemeralFuse(EphemeralFuse),
}

/// What is done to the provider's `volumes.ephemeral_fuse`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum EphemeralFuse {
    /// Print it, one line: `{"path":"…","capacity":…}`, the path
    /// resolved, absolute. `null` where it was deleted, or the
    /// `volumes` section is absent.
    Get,
    /// Set it, both members at once. Refused where the `volumes`
    /// section is absent — enable it first — where the capacity is
    /// `0`, and while the provider is running on the root. The folder
    /// is made by the provider at its start, and emptied then: nothing
    /// in it is kept from one start to the next, so a changed path
    /// moves nothing.
    Set {
        /// The folder: absolute, or relative to the root.
        #[arg(long)]
        path: std::path::PathBuf,
        /// The most every ephemeral FUSE serve's own layer may hold
        /// together, in BYTES: a plain integer, `34359738368` for
        /// 32 GiB.
        #[arg(long)]
        capacity: u64,
    },
    /// Remove it: the provider serves no ephemeral volume over FUSE.
    /// The folder is left as it is. Nothing changes where it is not
    /// set. Refused while the provider is running on the root.
    Delete,
}

/// What is done to the provider's `volumes.stores`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Stores {
    /// Print the stores, one line each, in the order they were
    /// added, which is the order of preference:
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
    #[command(name = "capacity_used")]
    CapacityUsed,
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
        /// what the volumes in the store reserve now: see
        /// `capacity_used`.
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

/// `diverge provider volumes fixed …`: the list, or one fixed volume
/// of it. With no index, the list is worked on — `get`, `add`,
/// `delete`. With a volume's index first, one setting of that volume
/// is — `diverge provider volumes fixed 0 authorized get`. The shape
/// of [`Daemons`](super::Daemons), the CLI's other list whose items
/// hold a list.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct Fixed {
    /// The index of the fixed volume whose setting is read or
    /// written, as `fixed get` prints it: `0` is the first. Left out
    /// for `get`, `add` and `delete`, which work on the list.
    #[arg(value_name = "INDEX")]
    pub index: Option<usize>,
    /// What to do.
    #[command(subcommand)]
    pub command: FixedCommand,
}

/// What is done to the provider's `volumes.fixed`, or to one of
/// them: see [`provider`](crate::cli::provider).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum FixedCommand {
    /// The list: print the fixed volumes, one line each, in the order
    /// they were added:
    /// `{"index":0,"name":"…","path":"…","bytes":…,"mode":"…","authorized":…}`,
    /// `authorized` the string `all` or the list of identities.
    /// Nothing is printed where there is none, or where the section
    /// is absent. The index is what a setting of one and `delete`
    /// take.
    Get,
    /// The list: add a fixed volume, after the last, listed to every
    /// identity. Refused where the section is absent — enable it
    /// first — where the name is listed already, and while the
    /// provider is running on the root.
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
    },
    /// The list: delete the fixed volume at an index: the entry,
    /// never the directory. Refused where the section is absent and
    /// while the provider is running on the root. Every one after it
    /// moves one index down.
    Delete {
        /// The volume's index, as `get` prints it: `0` is the first.
        #[arg(value_name = "INDEX")]
        index: usize,
    },
    /// One fixed volume: read who it is listed to, list it to every
    /// identity, or add or delete one of the identities it is listed
    /// to, its `authorized`.
    #[command(subcommand)]
    Authorized(Authorized),
}

/// What is done to one fixed volume's `authorized`: see
/// [`provider`](crate::cli::provider). It is the word `all`, or a
/// list of identities, worked on as every list of identities is.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Authorized {
    /// Print who the volume is listed to: the one line `"all"`, or
    /// the identities, one line each, in the order they were added:
    /// `{"index":0,"issuer":"…","subject":"…"}`. A list with nobody in
    /// it prints nothing. The index is what `delete` takes.
    Get,
    /// List the volume to every identity: `all`. Whatever identities
    /// were named are forgotten. Refused while the provider is
    /// running on the root: stop it first.
    Set {
        /// `all`: the one value there is to set. An identity is
        /// added, and never set.
        #[arg(value_name = "all", value_parser = ["all"])]
        value: String,
    },
    /// Add an identity, after the last. A volume listed to `all` is
    /// from then on listed to that identity alone. Refused while the
    /// provider is running on the root, and where the identity is
    /// listed already.
    Add {
        /// The identity's issuer, exactly as its discovery document
        /// names it.
        #[arg(long)]
        issuer: String,
        /// The identity's subject: the `sub` of its tokens.
        #[arg(long)]
        subject: String,
    },
    /// Delete the identity at an index. Deleting the last leaves the
    /// list empty, which lists the volume to nobody: `set all` is
    /// what lists it to everybody. Refused where the volume is listed
    /// to `all`, there being no list, and while the provider is
    /// running on the root. Every identity after it moves one index
    /// down.
    Delete {
        /// The identity's index, as `get` prints it: `0` is the first.
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
    /// Read or write the most every running container's disk, and
    /// every ephemeral volume a container mounts, may take together,
    /// `containers.container_overlay_disk`.
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
    /// delete one.
    #[command(subcommand)]
    Registries(Registries),
    /// Read where podman keeps its data, or move the data and write
    /// the new place.
    #[command(name = "storage_path", subcommand)]
    StoragePath(StoragePath),
    /// Read or write the most podman's image cache may hold.
    #[command(name = "image_cache_disk", subcommand)]
    ImageCacheDisk(ImageCacheDisk),
}

/// What is done to the provider's `podman.image_cache_disk`: see
/// [`provider`](crate::cli::provider).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum ImageCacheDisk {
    /// Print the number of bytes, as the provider reads it: the
    /// default, 32 GiB, where none was set.
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
    /// Print the registries, one line each, in their order:
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
    /// the order they were added:
    /// `{"index":0,"issuer":"…","subject":"…"}`.
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
    /// order they were added:
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
