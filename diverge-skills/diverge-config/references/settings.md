# Every setting

Every setting of the four programs, with its default and what it means. Each is read with `get` and changed
with the command shown in the `diverge-config` skill; nothing here is changed any other way. Byte sizes are
plain integers in BYTES.

Contents: [the daemon](#the-daemon) · [the provider](#the-provider) · [the CLI](#the-cli) ·
[the viewer](#the-viewer)

---

## The daemon

Read once, when the daemon starts. Commands: `diverge daemon <setting> …`.

| Setting | Default | Meaning |
|---|---|---|
| `port` | `14980` | TCP port the daemon accepts clients, providers and other daemons on: a WebSocket upgrade at any path, on every interface. One above a provider's, so the two do not contend on one host. |
| `identity` | none | `{issuer, subject}`: who the daemon is. The one identity whose token it presents on every connection, those it accepts and those it makes. It must be signed in under the daemon's root. **With none, the daemon completes no handshake.** It is not a list of trusted issuers: whom the daemon admits is its administrators, its accounts and its providers' records. |
| `admins` | none | List of `{issuer, subject}`: identities admitted as clients whether or not they are accounts, holding every grant. A fresh daemon has no account, so the first account is made by an administrator. |
| `postgres` | `local`, 1024 connections | The database the daemon keeps its records in. `local`: a Postgres the daemon runs itself, with `max_connections` (every container connection is one, so it is sized for many containers). `remote`: one it dials, by `url` (`postgres://user@host:port/database`, no password in it) and an optional `password`. Another database is another set of records. |
| `idle_seconds` | `10` | How long a container, an agent's or a tool's, may go unused before the daemon ends its run. The clock does not run while anything holds the container, and starts over when the last hold lets go and on every use. The record stays, and the next use starts the container again. |

What the daemon announces itself as on a provider, whether it accepts other daemons there, and whether it
hears direct messages there are not daemon settings. They are set per provider record, with
`diverge run providers add` and `diverge run providers edit`, while the daemon runs (skill `diverge-daemons`).

---

## The provider

Read once, when the provider starts. Commands: `diverge provider <setting> …`.

| Setting | Default | Meaning |
|---|---|---|
| `port` | `14979` | TCP port the provider accepts peers on: a WebSocket upgrade at any path, on every interface. |
| `identity` | none | `{issuer, subject}`: who the provider is. The one identity whose token it presents in every handshake. It must be signed in under the provider's root. **With none, the provider completes no handshake.** |
| `incoming` | none | List of `{issuer, subject}`: the identities that may dial the provider. A peer whose verified identity is not listed is closed in the handshake. **With none, nobody dials in.** A daemon that dials this provider must be listed. |
| `outgoing` | none | Peers the provider dials itself and keeps redialling, for a provider that cannot be dialled. Each is an `address` (`host:port`, or a URL with its own scheme) and the `{issuer, subject}` that must answer there. |

### podman

Used only by a provider with containers or images enabled. A provider with neither runs no podman and needs
none installed.

| Setting | Default | Meaning |
|---|---|---|
| `podman registries` | `registry.diverge.network`, `docker.io`, `ghcr.io`, `quay.io` | The registries the provider pulls from when an image's references name them. Each is a host: what a reference names before its first `/`; no scheme, no path; a port is allowed. The provider holds no credential for any, so only public images are pulled. A reference naming an unlisted registry is ignored. The daemon refuses a run unless at least one of the image's references names a listed registry. |
| `podman storage_path` | inside the config root | Where podman's data is kept: on Linux its image store, on macOS and Windows the `diverge-provider` machine. On Linux its filesystem must support project quotas (XFS `pquota`, or ext4 with project quota) or every container run fails. Changing it moves the data. |
| `podman image_cache_disk` | `34359738368` (32 GiB) | The most podman's image cache may hold. The provider removes images no running container uses, least recently run first, except images labelled `diverge.network/keep`. An image larger than this cannot be pulled. |

### containers (off by default)

| Setting | Default | Meaning |
|---|---|---|
| `containers` | disabled | Whether the provider runs containers. Disabled, every agent and tool run is refused as unsupported. |
| `containers container_overlay_disk` | `34359738368` (32 GiB) | The room containers write into, reserved up front: every running container's `disk`, plus the `overlay_disk` of every ephemeral volume a container mounts. A run that would take the total past it is refused. An ephemeral volume served over FUSE is **not** counted here; that is `volumes ephemeral_fuse`. |
| `containers memory` | `8589934592` (8 GiB) | The sum of every running container's `memory` never exceeds it; a run that would is refused. On macOS the podman machine is given exactly this much. |

### volumes (off by default)

| Setting | Default | Meaning |
|---|---|---|
| `volumes` | disabled | Whether the two lists below exist. The provider keeps volumes when either has something; with both empty it keeps none. |
| `volumes stores` | none | Places new volumes are created, in order of preference. Each is a `path` and a `capacity`: the most the volumes in it may reserve together (`0` is refused at start). With one or more, clients can create volumes of every mode. With none, no client can create a volume. |
| `volumes fixed` | none | Existing directories on the host that the provider offers as volumes as they are. Each: `name` (unique; never given to a created volume), `path` (must already be a directory, or the provider refuses to start), `bytes` (a declared size, not enforced), `mode` (`persistent`, `ephemeral` or `read_only`), and `authorized` (who it is listed to: `all`, the default, or a list of identities). Clients see one as `"provided": true` and can neither edit nor delete it. With fixed volumes and no store, only the modes they are in are offered, and nothing can be created. |
| `volumes ephemeral_fuse` | `<root>/provider/ephemeral_fuse`, `34359738368` (32 GiB): on whenever volumes are enabled | Where an ephemeral volume served over FUSE keeps what is written through it, and how much: a `path` and a `capacity` in bytes. Each serve reserves its `overlay_disk` from the capacity and is refused if it does not fit. **Deleted, the provider serves no ephemeral volume over FUSE** and does not advertise that it can; `set` turns it back on. It does not depend on containers being enabled. |

A fixed volume's `authorized` says which identities are shown it. `all`, the default, is every identity that
connects. A list is exactly the identities named, each by issuer and subject; anyone else is not shown the
volume and is not told it exists. An empty list is nobody.

### images (off by default)

| Setting | Default | Meaning |
|---|---|---|
| `images` | disabled | Whether the provider keeps a Store: the images clients build or push on it. Disabled, a build or push is refused, a listing is empty, and an image comes only from podman's cache, a listed registry, or the caller. |
| `images path` | inside the config root | Where the Store is kept. Changing it moves the Store. |
| `images capacity` | `34359738368` (32 GiB) | The most the Store may hold, every piece counted once. A push or build that would pass it is refused as insufficient capacity. Nothing a client owns is ever evicted. |

### messages (off by default)

| Setting | Default | Meaning |
|---|---|---|
| `messages` | disabled | Whether the provider relays direct messages between the daemons connected to it. Disabled, it relays none, and `providers daemons listen` and `message` are answered unsupported. |
| `messages messages` | `30` | The most sends one identity makes in one window. At least `1`. |
| `messages seconds` | `60` | The length of one window, in seconds. At least `1`. |

---

## The CLI

Read by every CLI command. Commands: `diverge cli daemons …`, `diverge cli default …`.

| Setting | Default | Meaning |
|---|---|---|
| `daemons` | one, named `local` | The daemons the CLI may dial, each with a name of the CLI's own (`--daemon <name>`). |
| `default` | `local` | Which daemon is dialled when a command names none. |
| a daemon's `address` | `ws://127.0.0.1:14980/` for `local` | Where the daemon is dialled: a WebSocket URL, or a `host:port`. |
| a daemon's `accounts` | none | The identities the CLI may dial that daemon as, each with a name of the CLI's own (`--account <name>`) and an `{issuer, subject}` signed in under this root. The identity must be an administrator or an account of that daemon. |
| a daemon's `default` | none | Which of its accounts is used when a command names none. With none, every `run` must pass `--account`. |
| a daemon's `identity` | none | `{issuer, subject}`: the daemon's **own** identity. Stated, the CLI connects only if that identity answers and proves its key. Unset, whichever identity answers with a valid token and key is accepted. |

The names (`local`, `me`) are the CLI's own labels. They are not the daemon's account names; the daemon knows
an account only by its identity.

---

## The viewer

Read once, when the viewer starts. Commands: `diverge viewer port|redial|daemons|default …`.

| Setting | Default | Meaning |
|---|---|---|
| `port` | `14981` | The port the viewer hears its stop on, on this host only: how `diverge viewer stop` reaches it. One above a daemon's. |
| `redial` | `5` | `never`, or a whole number of seconds (at least `1`) the viewer waits before dialling again after a connection ends or a dial fails. |
| `daemons` | one, named `local` | The daemons the viewer may dial, each with a name (`diverge viewer start --daemon <name>`). The viewer's own list, apart from the CLI's. |
| `default` | `local` | Which daemon a start dials when it names none. |
| a daemon's `address` | `ws://127.0.0.1:14980/` for `local` | Where the daemon is dialled: a WebSocket URL, or a `host:port`. |
| a daemon's `accounts` | none | The identities the viewer may dial that daemon as, each with a name (`--account <name>`) and an `{issuer, subject}` signed in under this root. The identity must be an administrator or an account of that daemon. What the viewer shows is what that account may see. |
| a daemon's `default` | none | Which of its accounts a start uses when it names none. **With none, a start must pass `--account`, or the viewer does not start.** |
| a daemon's `identity` | none | `{issuer, subject}`: the daemon's **own** identity. Stated, the viewer connects only if that identity answers and proves its key. Unset, whichever identity answers is accepted. |
