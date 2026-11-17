---
name: diverge-config
description: Configuring and operating a Diverge installation's root. Load it to find or choose the config root (`--config`, `DIVERGE_CONFIG`, `~/.diverge`); to read or change `daemon.yaml` one setting at a time (`diverge daemon port|idle_seconds|admins|identity|postgres …`, no JSON, refused while the daemon runs); to read or change the provider's port, identity, `incoming`, `outgoing` and registries the same way (`diverge provider port|identity|incoming|outgoing …`, `diverge provider podman registries …`); to move podman's data to another folder or drive (`diverge provider podman storage_path set`) or wipe it so the provider sets podman up again (`… storage_path delete`); to turn the provider's containers, volumes, image Store or messages on or off and set their sizes (`diverge provider containers|volumes|images|messages …`); to point the viewer at a daemon and an identity (`diverge viewer daemon address|issuer|subject|identity|redial get|set`); to read or change `cli.yaml` or `viewer.yaml` with `diverge <program> config get|set|patch`; to add a daemon or an account to the CLI; to sign an identity in, list or remove one (`diverge identities login`; the accounts at Diverge's own issuer, `diverge identity.diverge.network …`; `diverge run identities …` on the daemon's host); to start or stop the daemon or provider; to look up ports, admins, `idle_seconds`, registries, volume stores; or to work out what must be restarted after a change and why a connection or handshake is refused.
---

# Diverge: the root, the documents, identities, start and stop

Everything a Diverge program is told lives in one directory, the **root**: one YAML document per program, the
signed-in identities, logs and locks. This skill covers every command that works on the root — everything in
the `diverge` CLI that is not a request to a daemon's records — plus `diverge run identities`, the same
identity work done on a daemon's host.

If you have not loaded the `diverge` skill in this task, load it first.

## The root

```
<root>/
  daemon.yaml  provider.yaml  cli.yaml  viewer.yaml                  the documents (each optional)
  identities/<percent-encoded issuer>/<subject>.json                 signed-in identities (hold refresh tokens)
  identities/<percent-encoded issuer>/<subject>.key                  each identity's private key on this host
  identity.diverge.network/<issuer>.<id>.json                        signed-in accounts at Diverge's own issuer (hold GitHub / Google tokens)
  daemon.log  provider.log                                           a started server's output, truncated at each start
  daemon.lock provider.lock                                          held while that server runs; holds its process id
  daemon/  provider/  viewer/                                        each program's own state
  bin/                                                               the binaries, in a default installation
```

Which directory is the root:

| Program | Order |
|---|---|
| The CLI, daemon, provider, viewer | `--config <dir>`, else `DIVERGE_CONFIG`, else `~/.diverge` |

- The root is created if it does not exist. A mistyped `--config` path silently makes an empty root with default
  documents.
- `--config` is written before the command: `diverge --config <root> daemon port get`.
- A server started by the CLI is handed the CLI's root as `--config`, so it runs on the root the CLI found.
- One of each server per root (the lock enforces it). A second daemon on the same host is a second root with
  its own `daemon.yaml`, `cli.yaml` and identities.
- Identities are per root: a program uses only the identities signed in under its own root.

## Ports

| Port | Who | Set in |
|---|---|---|
| 14979 | provider (all interfaces) | `provider.yaml` `port` |
| 14980 | daemon (all interfaces) | `daemon.yaml` `port` — and repeated in `cli.yaml` and `viewer.yaml` addresses |
| random loopback | the daemon's own Postgres | picked at every daemon start |

## The documents at a glance

Every field of every document, with defaults, is in [references/documents.md](references/documents.md). Read it
before changing a field you have not changed before.

| Document | Read | Holds |
|---|---|---|
| `daemon.yaml` | at daemon start | `port`, `identity` (`{issuer, subject}`: who the daemon is), `admins`, `postgres` (`local` or `remote`), `idle_seconds`. Nothing per provider lives here: whether the daemon announces itself, accepts other daemons and hears messages is on each provider's record (`diverge run providers edit`, skill `diverge-daemons`) |
| `provider.yaml` | at provider start | `port`, `identity` (who the provider is), `incoming` (who may dial it), `outgoing` (peers it dials, each an `address` and the `identity` that must answer), `podman` (`registries`, `storage_path`, `image_cache_disk`; read only with `containers` or `images`), `containers` (absent: no containers; `container_overlay_disk`, `memory`), `volumes` (`stores`, `fixed`), `images` (the Store: `path`, `capacity`), `messages` (rate) |
| `cli.yaml` | by every CLI command | `default` daemon; `daemons.<name>`: `address`, `default` account, `accounts.<name>`: `issuer`, `subject` |
| `viewer.yaml` | at viewer start | `daemon`: `address`, `issuer`, `subject`, `redial` |

Rules common to all four:

- An absent file is the built-in defaults; nothing needs to exist for a `get` to work.
- **An unknown key is refused**, with the path to it. A typo never passes silently.
- No document holds a token. The only secret in any of them is `postgres.password` (daemon, remote kind). Do
  not print that document into logs or chat without need.
- The `issuer` string must be identical everywhere it appears — in logins, in `identity`,
  `admins`, `incoming`, `cli.yaml`, `viewer.yaml`. `https://identity.diverge.network` and
  `https://identity.diverge.network/` are different issuers.

## Reading and changing a document: `config get | set | patch`

The same three subcommands exist beneath two programs: `diverge viewer config` and `diverge cli config`. They
work on the file in the root and never contact a running program.

**The daemon and the provider have no `config` command.** `daemon.yaml` and `provider.yaml` are read and
changed one setting at a time, with plain arguments and no JSON: see "The daemon's settings" and "The
provider's settings" below.

### get — read, through jq

```bash
diverge cli config get                       # the whole document, one compact JSON line
diverge cli config get .default              # "local"
diverge cli config get '.daemons | keys[]'   # one line per value the filter yields
diverge viewer config get '.daemon.address'
```

The document is shown as JSON. Empty optional members are left out, so a filter over one may see `null`.

### set — rewrite the whole document with a jq program

The program runs with the document as input and must yield **exactly one value**, which becomes the whole new
document.

```bash
diverge cli config set '.default = "b"'
diverge viewer config set '.daemon = {address:"ws://127.0.0.1:14980/", issuer:"https://identity.diverge.network", subject:"github.12345.admin"}'
```

Pitfall: a member left out of the document is `null` to the program. Default a list or an object before
adding to it (`(.daemons // {})`), and set an identity whole: it is one `{issuer, subject}` object, and one
member of it alone is not a valid document.

### patch — change places with a JSON Patch (RFC 6902)

The argument is a list of operations written as YAML (JSON is YAML). Operations: `add`, `remove`, `replace`,
`move`, `copy`, `test`. Paths are JSON Pointers: `/default`, `/daemons/b/accounts/admin`; a list's `/-` appends (`add` only); inside
a key, `/` is written `~1` and `~` is `~0`. Operations apply in order, **all or none**.

```bash
diverge cli config patch '[{op: test, path: /default, value: local}, {op: replace, path: /default, value: b}]'
diverge cli config patch '[{op: add, path: /daemons/b, value: {address: "ws://127.0.0.1:14982/", default: admin, accounts: {admin: {issuer: "https://identity.diverge.network", subject: github.12345.admin}}}}]'
```

- `add` to an object makes or replaces the member; `add` to a list inserts at the index. `replace` and `remove`
  need the place to exist already.
- The parent must exist: a path beneath a member the document does not have fails with `nothing is at …`.
  Add the whole member instead.
- Use `test` first when a patch depends on a list index, so a document that changed underneath fails the patch
  rather than removing the wrong element.

### What `set` and `patch` do to the file

- The result is validated against the program's own type before anything is written. A refused result changes
  nothing: `Error: the document is not one its program takes: bogus: unknown field …`, exit 1.
- The file is rewritten whole, atomically, as the type's own YAML: **comments are lost** and keys come out in
  the type's order (the CLI's daemons and accounts keep theirs). Members equal to "none" are dropped.
- Validation is of shape only. A wrong port, an identity not signed in, or a path that does not exist is found
  when the program starts.
- The running program is not told. See "What needs a restart".
- Editing the YAML file by hand is equally valid, and is the only way to keep comments.

## The daemon's settings: `diverge daemon port`, `idle_seconds`, `admins`, `identity`, `postgres`

The only commands that read or change `daemon.yaml`. Each takes plain arguments and flags, never JSON.

```bash
diverge daemon port get                 # 14980 — the value the daemon runs with, the default where the file has none
diverge daemon port set 15000           # prints nothing
diverge daemon idle_seconds get         # 10
diverge daemon idle_seconds set 300
diverge daemon admins get               # {"index":0,"issuer":"https://identity.diverge.network","subject":"github.12345.admin"}   one line per administrator
diverge daemon admins add --issuer https://identity.diverge.network --subject github.12345.other
diverge daemon admins delete 0          # by the index `admins get` prints; those after it move down one
diverge daemon identity get             # {"issuer":"https://identity.diverge.network","subject":"github.12345.daemon-a"}, or null
diverge daemon identity set --issuer https://identity.diverge.network --subject github.12345.daemon-a
diverge daemon postgres get             # {"kind":"local","max_connections":1024}  or  {"kind":"remote","url":"…","has_password":true}
diverge daemon postgres set local                             # the daemon's own Postgres, 1024 connections
diverge daemon postgres set local --max-connections 200
diverge daemon postgres set remote --url postgres://diverge@db.example.com:5432/diverge --password s3cret
```

- `get` works whether the daemon is running or not.
- **Every `set`, `admins add` and `admins delete` is refused while the daemon is running** on the root, and nothing is written:
  `Error: the daemon is running on this root, as process <pid>, and nothing was written: stop it first, with`
  `` `diverge daemon stop` ``. Stop the daemon, set, then start it.
- `admins add` refuses an identity that is an administrator already; `admins delete` refuses an index the list
  does not reach. Delete from the highest index down when deleting several.
- Each one that writes rewrites the file whole, as the daemon's own YAML: comments are lost, and nothing is
  printed. Editing `daemon.yaml` by hand is equally valid.
- `postgres set` writes `postgres` whole and carries nothing over: `set local` without `--max-connections`
  writes 1024, and `set remote` without `--password` writes no password, whatever the file had. `--url` is
  required for `remote`, holds no password, and is not dialled or checked. `postgres get` never prints the
  password. A password given as a flag is in the shell's history.
- `identity set` takes both flags and replaces the identity whole. It cannot remove the identity; delete the
  `identity` member from `daemon.yaml` by hand for that. It does not check that the identity is signed in.
- The whole document is printed by reading the file: `cat <root>/daemon.yaml` (absent: every setting is at
  its default).

## The provider's settings: `diverge provider port`, `identity`, `incoming`, `outgoing`, `podman registries`, `podman storage_path`, `podman image_cache_disk`

Seven settings of `provider.yaml` have their own commands, by the daemon's rules: plain arguments and flags,
`get` always answered, every write refused while the provider is running on the root (`diverge provider stop`
first), nothing printed by a write, comments lost.

```bash
diverge provider port get               # 14979
diverge provider port set 15001
diverge provider identity get           # {"issuer":"…","subject":"…"}, or null
diverge provider identity set --issuer https://identity.diverge.network --subject github.12345.provider
diverge provider incoming get           # {"index":0,"issuer":"…","subject":"…"}   one line per identity accepted
diverge provider incoming add --issuer https://identity.diverge.network --subject github.12345.daemon-b
diverge provider incoming delete 0
diverge provider outgoing get           # {"index":0,"address":"…","issuer":"…","subject":"…"}   one line per peer dialled
diverge provider outgoing add --address relay.example.com:14979 --issuer https://identity.diverge.network --subject github.12345.relay
diverge provider outgoing delete 0
diverge provider podman registries get  # {"index":0,"host":"docker.io"}   one line per registry; the default three where the file lists none
diverge provider podman registries add registry.example.com:5000
diverge provider podman registries delete 2
diverge provider podman storage_path get                    # C:\Users\me\.diverge\provider\podman_data   the resolved, absolute path
diverge provider podman storage_path set D:\diverge\podman  # moves podman's data there, then writes the path
diverge provider podman storage_path set D:\diverge\podman --no-transfer   # writes the path, moves nothing
diverge provider podman storage_path delete                 # deletes podman's data at the current path; the next start makes it anew
diverge provider podman image_cache_disk get                # 34359738368   bytes; the default (32 GiB) where the file has none
diverge provider podman image_cache_disk set 68719476736    # 64 GiB, as a plain number of bytes
diverge provider containers enabled                         # true or false: does this provider run containers
diverge provider containers enable                          # adds the `containers` section with its defaults
diverge provider containers disable                         # removes the section, and its settings with it
diverge provider containers container_overlay_disk get      # 34359738368, or null while containers are disabled
diverge provider containers container_overlay_disk set 68719476736
diverge provider containers memory get                      # 8589934592, or null while containers are disabled
diverge provider containers memory set 17179869184
diverge provider volumes enabled                            # true or false: is the `volumes` section in the file
diverge provider volumes enable                             # adds the section, with nothing in it
diverge provider volumes disable                            # removes the section and both its lists
diverge provider volumes fixed get                          # {"index":0,"name":"datasets","path":"/mnt/datasets","bytes":…,"mode":"read_only"}   one line each; nothing when none
diverge provider volumes fixed add --name datasets --path /mnt/datasets --bytes 107374182400 --mode read_only
diverge provider volumes fixed delete 0                     # removes the entry, never the directory
diverge provider volumes stores get                         # {"index":0,"path":"provider/volumes","capacity":107374182400}   one line each; nothing when none
diverge provider volumes stores add --path provider/volumes --capacity 107374182400
diverge provider volumes stores capacity_used               # {"index":0,"capacity":107374182400,"used":21474836480,"free":85899345920}   one line per store
diverge provider volumes stores edit 0 --capacity 214748364800   # just the number; refused if below `used`
diverge provider volumes stores edit 0 --path D:\divergeolumes # MOVES the store and every volume in it, then writes the path
diverge provider volumes stores delete 0                    # DELETES the folder and every volume in it, then the entry
diverge provider images enabled                             # true or false: does this provider keep an image Store
diverge provider images enable                              # adds the `images` section with its defaults
diverge provider images disable                             # removes the section; the folder and its images are left on disk
diverge provider images path get                            # the resolved, absolute path, or null while disabled
diverge provider images path set D:\diverge\images          # MOVES the Store there, then writes the path
diverge provider images path set D:\diverge\images --no-transfer   # writes the path, moves nothing: an empty Store at the new place
diverge provider images path delete                         # DELETES the Store's folder and every image in it; the section stays
diverge provider images capacity get                        # 107374182400, or null while disabled
diverge provider images capacity set 214748364800           # refused if below `used`
diverge provider images capacity_used                       # {"capacity":…,"used":…,"cached":…,"free":…}, or null while disabled
diverge provider messages enabled                           # true or false: does this provider relay direct messages
diverge provider messages enable                            # adds the `messages` section: 30 messages per 60 seconds
diverge provider messages disable                           # removes the section
diverge provider messages messages get                      # 30, or null while disabled: sends per identity per window
diverge provider messages messages set 120
diverge provider messages seconds get                       # 60, or null while disabled: the window, in seconds
diverge provider messages seconds set 30
```

- `incoming` works as the daemon's `admins` does: `add` appends and refuses an identity listed already;
  `delete` takes the index `get` prints and refuses one the list does not reach.
- `outgoing add` takes all three flags. It refuses only the same identity at the same address; the same
  identity at another address is another peer.
- `identity set` cannot remove the identity; delete the member from the file by hand.
- `podman registries add` takes the host as its one argument (no scheme, no path) and refuses one listed
  already. Deleting every registry leaves an empty list, not the default three.
- `podman image_cache_disk set` takes BYTES as a plain integer (no `GiB` suffix). It caps podman's own image
  storage, not the provider's Store (`images.capacity`). Lowering it deletes nothing at once: the provider
  trims the cache down at its next container deploy.
- **`containers`** is on when its section is in the file and off when it is not (a fresh provider has it
  off). `enable` on an already enabled provider changes nothing and keeps its settings. `disable` forgets
  `container_overlay_disk` and `memory`; a later `enable` starts from the defaults (32 GiB, 8 GiB).
- A `get` of either containers setting prints `null` while containers are disabled. A `set` is refused then:
  ``Error: `containers` is disabled, and nothing was written: enable it first, with `diverge provider containers enable` ``.
- Enabling containers makes the provider need podman and the container proxy binary at its next start.
  `memory` also sizes the podman machine on macOS; elsewhere it only caps what containers may reserve.
- **`volumes`**: the section holds two lists, `stores` (where volumes are created) and `fixed` (volumes the
  provider provides as they are). The provider keeps volumes when either list has something; a section with
  both empty keeps none, the same as no section. So `volumes enabled` says the section is in the file, not
  that the provider keeps volumes.
- `volumes fixed get` prints nothing when the section is absent or the list is empty; that is not an error.
  `fixed add` and `fixed delete` are refused while the section is absent (`volumes enable` first).
- `fixed add` takes `--name`, `--path`, `--bytes` (plain bytes, a declared size, not measured), `--mode`
  (`persistent`, `ephemeral` or `read_only`) and optionally `--authorize-hook`. It refuses a name already
  listed. The path must already be a directory by the time the provider starts, or it refuses to start.
- A fixed volume shows to clients as `"provided": true` and cannot be edited or deleted by them.
- **`volumes stores`** is a list like the others, plus `edit`, because a store is a folder holding every volume
  created in it (each a large, mostly empty image file):
  - `stores add` only writes the entry; the provider makes the folder at its next start. It refuses a path
    that is already a store's.
  - `stores capacity_used` shows, per store, `capacity`, `used` and `free` in bytes. `used` is what the volumes
    in the store *reserve* (the sum of their sizes), which is what counts against the capacity; it is not
    how much disk the folder takes, since a volume reserves its whole size however little it holds.
  - `stores edit <index> --capacity <BYTES>` only writes the number. It is refused when the number is less
    than `used`: ``the volumes in store 0 reserve … bytes, and its capacity cannot be …, which is less``.
    Shrink or delete volumes first.
  - `stores edit <index> --path <PATH>` **moves the store**, as `podman storage_path set` moves podman's
    data: a rename on the same drive (instant, no extra space), or a checked copy to another drive with the
    old copy deleted only after the path is written. The new path must not exist or must be empty. If the
    move fails, everything is put back and nothing is written. Interrupted? Run the same command again.
  - `stores delete <index>` **deletes the folder and every volume in it**, then the entry. There is no undo
    and no prompt. A failure midway writes nothing; run it again to finish.
  - Before a move or a delete the podman machine is stopped if it is running (macOS, Windows); on Linux the
    command is refused while a volume is still mounted under `<root>/provider/run/mounts`.
  - On macOS a store moved outside the folders the machine was made seeing makes the provider remake its
    machine at the next start, which empties podman's image cache.
  - The daemon's tags for the volumes are unaffected by a move. After a delete, the tags of the deleted
    volumes are cleaned up the next time the daemon connects to the provider.
- **`images`** is the provider's Store, where clients build and push images. It is on when its section is
  in the file.
  - `images enable` adds the section with its defaults: path `provider/images` (under the root) and a
    capacity of 32 GiB (`34359738368`). On an already enabled provider it changes nothing and keeps the
    settings. Change them afterwards with `images path set` and `images capacity set`.
  - `images disable` only removes the section. The Store's folder and images stay on disk, and enabling again
    with the same path finds them.
  - `images path set` moves the Store exactly as `podman storage_path set` moves podman's data (rename on one
    drive, checked copy across drives, all or nothing, rerun to resume). The new path must not exist or must be
    empty. `--no-transfer` writes the path and leaves the old Store where it is.
  - `images path delete` deletes the folder and every image in it, with no undo and no prompt. The section and
    its path stay, so the provider starts with an empty Store there.
  - `images capacity_used`: `used` is what the images clients **own** take (each blob counted once); `cached`
    is what is kept besides (images clients supplied to a run), which the provider removes whenever it needs
    room; `free` is `capacity` minus `used`.
  - `images capacity set` is refused below `used`. Cached bytes do not block it.
  - A `get` or `capacity_used` prints `null` while `images` is disabled; a `set` or `path delete` is refused
    then.
  - Enabling `images` makes the provider need podman at its next start (it builds with it). On macOS a Store
    moved outside the folders the machine sees makes the provider remake its machine at the next start.
- **`messages`** is the provider relaying direct messages between the daemons connected to it. It is on when
  its section is in the file. `messages enable` adds it with the default rate, **30 messages per sending
  identity in every 60 seconds**; `messages disable` removes it. `messages messages` is the count and
  `messages seconds` the window; each is `1` or more (a `0` is refused), prints `null` on `get` while messages
  are disabled, and is refused on `set` then.
- These commands are the only way the CLI changes `provider.yaml`; there is no `diverge provider config`.
  To see the whole file, read it: `cat <root>/provider.yaml` (absent: every setting is at its default).
  Editing the file by hand is equally valid, with the provider stopped.

### Moving podman's data: `storage_path set`

`podman.storage_path` is where the provider's podman keeps everything: on Linux its image store, on macOS and
Windows the whole `diverge-provider` machine (its disk can be tens of gigabytes). `set <path>` moves it and
then writes the path; it prints nothing.

- **Stop the provider first.** The command holds the provider's lock from start to end, so the provider cannot
  be started while the data is between two places.
- **Same drive or filesystem: a rename.** Instant, no extra space, however large the machine.
- **Another drive or filesystem: a copy, checked, then the old copy deleted.** There is no other way to cross
  one. It needs room for all of it at the new place (checked first) and takes as long as the copy.
- **All or nothing.** If anything fails, what was moved is put back and the path is not written. The error
  starts `podman's data was not moved, and the path was not written:`.
- **The machine is stopped for you** if it is running (macOS, Windows), even if the provider was killed and
  left it up.
- **The new path must not exist, or be an empty folder**, and must not be inside the old one (or the old one
  inside it).
- **An old path with nothing in it** (a provider that never ran containers) is just a path change.
- **A provider with no `containers` and no `images` section** uses no podman: the path is written, nothing is
  moved.
- **Interrupted (power loss, a kill)?** Run the same `set` again. It picks up where it stopped.
- Linux: run it as root, as the provider runs. It is refused if anything is still mounted under the old path,
  or if podman's store there holds a container, pod or volume (remove them first).
- Windows: it needs a WSL that has `wsl --manage --move` (2.3.11 or later; `wsl --update`). Every WSL
  distribution whose disk is under the old path is moved, including a podman machine someone else made there.
- `--no-transfer` writes the path and leaves the old data where it is. On Windows it is refused while the old
  machine exists, because the provider would refuse to start beside it: transfer instead, or run
  `storage_path delete` first.
- If the error is `…but the old path could not be cleared`, the move succeeded and the path is written; delete
  what is left at the old path yourself.

### Starting over: `storage_path delete`

`diverge provider podman storage_path delete` deletes everything podman keeps at the current storage path, so
the next `provider start` sets it all up again: an empty image store on Linux, a brand-new machine on macOS
and Windows. It prints nothing and does not change the path.

- **Stop the provider first**; it holds the provider's lock like `set`.
- **Destructive, with no undo and no prompt.** Every image built or pulled into that store or machine is gone
  and must be built or pulled again. The provider's own Store (`images.path`) and volumes are separate folders
  and are not touched.
- On Windows it also removes what points at the folder from outside: the machine's WSL distribution, its
  description under `~/.config`, and its entries in podman's connections file. Deleting the folder by hand
  does not do that, and leaves the provider unable to start.
- Everything in the folder goes, including a podman machine someone else made with its data there.
- **Changing the path on Windows without carrying the data:** `delete`, then `set <path> --no-transfer`.
- A failure midway leaves it part deleted; run `delete` again to finish.

## The viewer's settings: `diverge viewer daemon …`

`viewer.yaml` has one section, `daemon`: which daemon the viewer dials, and as which identity. Each member has
its own command, with plain arguments. `diverge viewer config get|set|patch` still works on the whole file.

```bash
diverge viewer daemon address get        # ws://127.0.0.1:14980/   the default: a daemon on this host
diverge viewer daemon address set ws://10.0.0.5:14980/
diverge viewer daemon issuer get         # an empty line until one is set
diverge viewer daemon issuer set https://identity.diverge.network
diverge viewer daemon subject set github.12345.admin
diverge viewer daemon identity get       # null, or {"issuer":"…","subject":"…"}: the daemon's OWN identity
diverge viewer daemon identity set --issuer https://identity.diverge.network --subject github.12345.daemon-a
diverge viewer daemon redial get         # 5, or never
diverge viewer daemon redial set never
```

- Every member has a default, so every `get` answers and every `set` works even when `viewer.yaml` does not
  exist or has no `daemon`: address `ws://127.0.0.1:14980/`, issuer and subject empty, no `identity`, redial `5`.
- **The viewer does not start while `issuer` or `subject` is empty.** Sign the identity in first
  (`diverge identities login`), then set both.
- `issuer`/`subject` are who the viewer dials **as**. `identity` is who the daemon must **be**; leave it unset
  to accept whichever daemon answers at the address. `identity set` cannot clear it; delete the member by hand
  or with `viewer config`.
- `redial set` takes `never` or a whole number of seconds (`1` or more).
- No lock is involved and the app need not be closed: a change is read at the viewer's next start.
- A `set` prints nothing and rewrites the file (comments are lost).

## Accounts at Diverge's own issuer: `diverge identity.diverge.network`

`https://identity.diverge.network` is Diverge's hosted issuer. Every subject there is `<issuer>.<id>.<name>`:

- `<issuer>` is `github` or `google`: who issued the owner's own sign-in. In these commands "issuer" always
  means GitHub or Google, never `identity.diverge.network` itself.
- `<id>` is the owner's permanent user id there.
- `<name>` is chosen by the owner: 1 to 64 characters of `a-z`, `0-9`, `-`, `_`.

An **account** is `<issuer>.<id>`. It owns every subject that starts with it. Sign an account in once per
root; after that, signing any of its subjects in needs no person (see the next section).

```bash
diverge identity.diverge.network accounts login github
# {"user_code":"ABCD-EFGH","verification_uri":"https://github.com/login/device","verification_uri_complete":null,"expires_in":899,"browser_opened":true}
diverge identity.diverge.network accounts list
# {"account":"github.12345","issuer":"github","id":"12345"}        one line per account
diverge identity.diverge.network accounts info github 12345
# {"account":"github.12345","issuer":"github","id":"12345","info":{…}}
diverge identity.diverge.network subjects list github 12345
# {"subject":"github.12345.daemon-a","name":"daemon-a","refresh_tokens":2}     one line per subject signed in somewhere
diverge identity.diverge.network refresh-tokens list github 12345 daemon-a
# {"id":"…","created":…,"ip":"…","user_agent":"…","last_used":…,"last_ip":"…"}   one line per place it is signed in
diverge identity.diverge.network refresh-tokens end github 12345 daemon-a <id>
# {"ended":"<id>"}
```

| Command | What it does |
|---|---|
| `accounts login <issuer>` | Starts a device login at GitHub or Google, opens that page in this host's browser, prints the code, and **ends**. A person enters the code on the page. A background process saves the account once it is approved. `--no-browser` opens nothing. |
| `accounts list` | The accounts signed in under this root. Reads no token, asks nobody. |
| `accounts info <issuer> <id>` | What GitHub or Google says about that user right now. The `info` object is the issuer's own and differs between the two. Never cached. |
| `subjects list <issuer> <id>` | The account's subjects that are signed in somewhere, with how many places each. |
| `refresh-tokens list <issuer> <id> <name>` | Where one subject is signed in: one entry per sign-in, with the address and program that started it and when it was last used. |
| `refresh-tokens end <issuer> <id> <name> <refresh-token-id>` | Signs one of them out. Tokens it already holds last until they expire (15 minutes). |

Facts and pitfalls:

- **`accounts login` needs a person once.** Read `user_code` from the printed line and give it to the person.
  If `browser_opened` is `false`, also give them `verification_uri`. `true` means a browser was started, not
  that anyone saw the page, so the address is always in the line.
- **`accounts login` prints no outcome.** Check with `accounts list`: the account appears once the code was
  approved. If it was denied or expired, nothing is written: run it again.
- The account's id is not chosen: it is whoever approved the code. One person signing in twice replaces the
  same account.
- An account whose issuer no longer accepts its token answers an error that says it is signed out. Run
  `accounts login` again; the file is replaced.
- A subject is listed only while it is signed in somewhere. Ending its last refresh token makes it vanish
  from `subjects list`; signing in under the same name brings the same subject back.
- No command here prints a token.
- `DIVERGE_IDENTITY_URL` points these commands at another copy of the service. Unset, they use
  `https://identity.diverge.network`.

## Identities on this host: `diverge identities`

An identity is one subject at one issuer, signed in once and kept signed in by its refresh token. All programs
on a root share `<root>/identities/`. No command prints a token or a private key.

**Every identity has a key on its host.** `<subject>.key` is a P-256 private key that never leaves the host.
Each request for tokens proves the key to the issuer (a DPoP proof, RFC 9449), the issuer binds the access
token to it, and every connection made as the identity is signed with it. A token copied off the wire or out
of a log cannot be used by anyone else. The key is made by itself, at the login or the first time a token is
needed, and there is no command for it. The same identity signed in on another host has a different key there.

The issuer binds the sign-in to that key: only the key it was signed in with can refresh it. So a key that is
lost or replaced means logging in again. To replace a key you believe was copied, `remove` the identity and
log in again.

### login

```bash
# At Diverge's own issuer: nothing to approve, the command does it and waits.
diverge identities login --issuer https://identity.diverge.network --subject github.12345.admin --client-id diverge
# → {"issuer":"https://identity.diverge.network","subject":"github.12345.admin"}     signed in

# At any other issuer: a code to approve there.
diverge identities login --issuer https://auth.example.com --subject alice --client-id my-client
# → {"user_code":"…","verification_uri":"…","verification_uri_complete":"…","expires_in":…}   then the command ends
```

The login is the device authorization grant (RFC 8628), and it is the only login there is. It behaves in one
of two ways, decided by `--issuer` alone:

| | `--issuer https://identity.diverge.network` (exactly) | Any other issuer |
|---|---|---|
| Who approves the code | The command itself, as the account that owns the subject | A person, at the issuer |
| When the command ends | When the identity is signed in (a couple of seconds) | As soon as the code is printed |
| What it prints | `{"issuer","subject"}`, the identity signed in | The code and where to approve it |
| How to know it worked | The printed line; an error otherwise | `diverge identities list`, later |
| What it needs first | The subject's account signed in under this root | Nothing |

| Flag | Meaning |
|---|---|
| `--issuer` (required) | The issuer, exactly as its discovery document names it. `https`, or `http` on loopback. It must offer a device authorization endpoint. |
| `--subject` (required) | The subject to sign in as. An approval as any other subject writes nothing. |
| `--client-id` (required) | The public client to sign in as. At `identity.diverge.network` it is `diverge`. |
| `--scope` | Space-separated; `openid` and `offline_access` are added if missing. Absent, those two. |
| `--resource` | RFC 8707 `resource` for the access token. Rarely needed. |

At `https://identity.diverge.network`:

- **The issuer string must match exactly.** A trailing slash, or any other address, is treated as another
  issuer and takes the code-and-background path.
- **The subject must be `<issuer>.<id>.<name>`.** It is split at its first two periods. Anything else is
  refused before anything is contacted, with the form it should have.
- **The account `<issuer>.<id>` must be signed in under this root.** If it is not, the error says so and gives
  the fix: `diverge identity.diverge.network accounts login <issuer>`, then run the login again.
- **A new subject and an existing one are the same command.** Signing in under a name never used before is how
  a subject begins. Signing in again under a used name adds another sign-in for the same subject.
- The name must be 1 to 64 of `a-z`, `0-9`, `-`, `_`, or the issuer refuses the approval.

At any other issuer:

- **The command ends as soon as the code is printed.** It does not wait for the approval. A background
  process of the CLI keeps polling the issuer until the code is approved, denied or expires, and prints
  nothing.
- **Check the outcome with `diverge identities list`.** The identity appears once the login was approved as
  the subject named. If the code was denied or expired, or approved as another subject, nothing is written
  and nothing is reported: run `login` again.
- No browser is opened. Approve the printed code at the issuer.

At every issuer:

- A login replaces an identity already there, signed out or not. Logging in again is how a signed-out identity
  is repaired.
- A second `login` of the same identity while one is still waiting waits its turn behind the first.
- The issuer must be reachable (its discovery document is fetched).
- An identity whose refresh token was ended, or went unused past the issuer's refresh-token lifetime (30 days
  at `identity.diverge.network`), is *signed out*: its file stays, handshakes fail, and a new login fixes it.
- **The issuer must support DPoP with ES256.** `identity.diverge.network` does. A login at an issuer whose
  discovery document does not list `ES256` under `dpop_signing_alg_values_supported` fails saying so, before
  anything is asked of it. An issuer that lists it but answers access tokens that are opaque, or not bound to
  the key, fails later with "answered an access token that is not bound to the identity's key".
- An identity signed in before keys existed is bound to its key at its next token refresh, with no new login.


### list, remove

```bash
diverge identities list
# {"issuer":"https://identity.diverge.network","subject":"github.12345.admin"}      one line per identity; signed-out ones included
diverge identities remove --issuer https://identity.diverge.network --subject github.12345.old
# {"removed":{"issuer":"…","subject":"github.12345.old"}}
```

`remove` deletes the identity's key with it.

Removing the identity a running daemon or provider presents (its `identity`) breaks every later handshake
of that program. Removing one that `cli.yaml` or `viewer.yaml` names breaks that account.

## Identities on the daemon's host: `diverge run identities`

The same store, on the host of the daemon you dial — for when the daemon runs somewhere you have no shell.
These are requests to the daemon: they need an account with grants over `identities` (an admin has them), take
the usual `run` flags and `relay`, and answer `"forbidden"` otherwise.

```bash
diverge run identities list                                   # {"listed":[{"issuer":…,"subject":…,"tags":[…]}, …]}
diverge run identities list --issuer https://identity.diverge.network   # also --subject; body: issuers, subjects, all_tags, any_tags
diverge run identities get   '{"identity":{"issuer":"https://identity.diverge.network","subject":"github.12345.daemon-a"}}'                  # {"found":{…}} | "not_found"
diverge run identities tag   '{"identity":{"issuer":"https://identity.diverge.network","subject":"github.12345.daemon-a"},"tags":["prod"]}'   # "tagged" | "not_found"
diverge run identities untag '{"identity":{"issuer":"https://identity.diverge.network","subject":"github.12345.daemon-a"},"tags":["prod"]}'   # "untagged" | "not_found"
diverge run identities login --issuer https://identity.diverge.network --subject github.12345.daemon-a --client-id diverge
# {"signed_in":{"issuer":"https://identity.diverge.network","subject":"github.12345.daemon-a"}}
```

- `list` is one snapshot (`{"listed":[…]}`), not a stream; there is no `--watch`. Tags exist only so grants can
  reach some identities and not others (skill `diverge-access`).
- `login` takes `--issuer --subject --client-id [--scope --resource]`, or the same members in a BODY
  (`issuer`, `subject`, `client_id`, `scope`, `resource`). The daemon starts a device login on its own host.
  What happens next depends on the issuer, exactly as for `diverge identities login` above.
- **At `https://identity.diverge.network`** the command approves the code itself and waits. It prints
  `{"signed_in":{…}}`, or why not. The daemon's host does the sign-in, but the approval uses the account
  signed in under **the CLI's own root**, on the machine where you run the command. The same rules apply:
  exact issuer string, subject `<issuer>.<id>.<name>`, account signed in here first.
- **At any other issuer** the command prints `{"pending":{"user_code",…}}` and ends. A background process of
  the CLI keeps the login alive until the code is approved, denied or expires. Check the outcome with
  `diverge run identities list`; it is never printed.
- Where the login could not begin, either way: `"held"` (another login or reader holds that identity's lock
  right now; try again), `"forbidden"`, or `{"error":…}` (issuer unreachable, or it has no device
  authorization endpoint).
- Signing an identity in does not make the daemon present it: which identity the daemon presents is
  still `daemon.yaml` `identity`, read at start.

## Start and stop: `diverge daemon|provider start|stop`

```bash
diverge provider start > provider-start.out 2>&1; cat provider-start.out   # started provider, process <pid>
diverge daemon start   > daemon-start.out   2>&1; cat daemon-start.out     # started daemon, process <pid>

diverge daemon stop      # stopped daemon      (or: daemon is not running — exit 0)
diverge provider stop
```

**Never pipe a `start`** (no `|`, no `$(…)`): the detached server inherits the pipe and the shell hangs.
Redirect to a file, as above.

`start`:

- Runs the program's binary from the directory the `diverge` executable is in, detached, with `--config <root>`.
  It returns once the program holds the root's lock.
- Refuses when one already runs on this root: `the daemon is already running on this root, as process <pid>`.
- If the program dies first, the error carries its whole log. Usual causes: the port is taken, a document it
  refuses, a root it cannot use, and for a provider with a `containers` or `images` section podman or `ssh` missing from `PATH` (a provider with neither needs no podman). The same text is in `<root>/<program>.log`.
- The log file is truncated at every start. A program that starts well prints nothing.
- The first start on Windows or macOS of a provider with a `containers` or `images` section creates and boots a podman machine and can take minutes.
- On Windows and macOS that machine (`diverge-provider`) runs only while the provider does: every start of such a provider starts it if it is stopped, and `diverge provider stop` stops it after the provider's containers are removed, which adds some seconds to the stop. A provider that was killed leaves the machine running; the next start finds it running and uses it.

`stop`:

- Asks the program to stop over its own port from this host, and returns once the lock is free — which is the
  whole teardown done. A daemon stop ends every container run and its Postgres; a **provider stop removes every
  container it ran**.
- `daemon stop` finds the port from **`cli.yaml`: the address of the default daemon**, not from `daemon.yaml`.
  That address must be on this host (`127.0.0.1`, `localhost`, `::1`), else `… is not this host`. So each root's
  `cli.yaml` default must point at that root's own daemon, and must be updated when `daemon.yaml` `port`
  changes. `provider stop` reads the port from its own document.
- `nothing is listening on port …: the program is not running, or is still starting` means the lock is held but
  the port is not open yet (or `cli.yaml` names the wrong port). Wait and retry, or fix the address.
- A stop is honoured only from the same host; there is no remote stop.

Order. Start: provider, then daemon (the daemon connects to the provider; a daemon
redials its providers every 5 seconds, so a late provider is picked up). Stop in reverse: daemon, then
provider. With several roots, stop every daemon before the provider they share.

There is no `restart`: stop, then start. There is no `viewer start`: the world viewer is a desktop app started
outside the CLI.

## What needs a restart

No server watches its document. A change takes effect as follows.

| Change | Takes effect |
|---|---|
| Anything in `daemon.yaml` | At the next daemon start: `diverge daemon stop`, then `start`. Agents and tools keep their records; running containers are ended and start again on next use. |
| `daemon.yaml` `port` | Daemon restart **and** update every `cli.yaml` and `viewer.yaml` address that names it (before the stop, `cli.yaml` must still hold the old port, or the stop cannot find the daemon). Other daemons and providers that dial this one by address need their records edited. |
| `daemon.yaml` `postgres` | Daemon restart. A different database is a different set of records: the daemon comes up with whatever that database holds. |
| Anything in `provider.yaml` | At the next provider start. Stopping the provider removes every container it ran; connected daemons show it `disconnected`, redial every 5 seconds, and start containers again on next use. |
| `provider.yaml` `podman.storage_path` | Changed with `diverge provider podman storage_path set`: the data is moved, and the provider starts on it as before. Changed by hand or with `--no-transfer`: provider restart onto a fresh, empty podman store (or a new machine), and every image built into the old one must be built again. |
| `cli.yaml` | The next CLI command. Nothing to restart. |
| `viewer.yaml` | The next start of the viewer app. |
| `diverge identities login` of an identity a running program already uses | No restart: programs read the identity's file whenever they need a token. |
| A program's `identity` changed, or a *new* entry in `admins` or `incoming` | Restart that program. Its own `identity` must be signed in under its root first. |
| Records in the daemon (providers, accounts, roles, agents, …) | Immediate; those are `diverge run` requests, not configuration. |

Before restarting anything, say what it will interrupt and make sure the user asked for it.

## Recipes

**Add an account to the CLI and make it the default.**

```bash
diverge cli config patch '[
  {op: add, path: /daemons/local/accounts/me, value: {issuer: "https://identity.diverge.network", subject: github.12345.admin}},
  {op: add, path: /daemons/local/default, value: me}]'
diverge run info        # {"info":{"version":"…"}}
```

**Add a second daemon to the CLI** — the `cli config patch` example with `/daemons/b` above — then
`diverge run --daemon b info`.

**Let a daemon use a provider.** In the provider's root: the daemon's identity goes into `incoming`, then the
provider restarts. In the daemon's root nothing changes; the provider record is added with
`diverge run providers add` (skill `diverge-daemons`).

**Keep containers up longer.** `diverge daemon stop`, `diverge daemon idle_seconds set 300`, `diverge daemon start`.

**Give the provider volumes.** With the provider stopped: `diverge provider volumes enable`, then
`diverge provider volumes stores add --path provider/volumes --capacity 107374182400`, then start it. (Volumes themselves: skill `diverge-storage`.)

## When something refuses

| Message | Cause | Fix |
|---|---|---|
| `no account was named, and the daemon … has no default` | `cli.yaml` has no default account for that daemon (this stops even `request-schema`) | Add an account and `default`, or pass `--account` |
| `no daemon is named …` / `the daemon … has no account named …` | `--daemon` / `--account` name nothing in `cli.yaml` | `diverge cli config get` |
| `the daemon could not be dialled: …` | Daemon down, or wrong `address` in `cli.yaml` | `diverge daemon start`, or fix the address |
| `the handshake did not complete: …` | The account's identity is not signed in under this root, is signed out, the issuer is unreachable or writes no key into its tokens, the daemon has no `identity` or its identity is not signed in under the daemon's root, or `cli.yaml` states a daemon `identity` that is not the one that answered | `diverge identities list`; log in again; check `daemon.yaml` `identity` and the daemon entry's `identity` in `cli.yaml` |
| `the daemon closed the connection without answering: the account may be none of the daemon's` | The identity is neither in `admins` nor an account on that daemon | Add it to `admins` and restart, or have an admin create the account |
| `the document is not one its program takes: <path>: …` | A `set`/`patch` result the type refuses | Fix the key or value at that path; nothing was written |
| `the patch could not be applied: #<n> <op>: nothing is at …` | The path, or its parent, does not exist | Add the parent member whole |
| `the jq program: …` | The filter does not parse, or failed on a missing member | Default absent members with `//` |
| `` `…yaml` could not be read: … `` | The file on disk does not parse as its type (hand edit) | Fix the file by hand; `config set` cannot run on it |
| A provider record stays `disconnected` | Daemon's identity missing from `provider.yaml` `incoming`; no `identity` in `daemon.yaml` or `provider.yaml`, or it is not signed in; the provider's `identity` is not the one the record names; provider down | Fix, restart the provider; the daemon redials by itself |
| Wrong documents are being read | `DIVERGE_CONFIG` set | `--config <root>` explicitly |

## Not verified

- On Linux the provider must run as root while the CLI and daemon usually do not; how one root's `identities/`
  is shared between them (or whether the provider should have a root of its own) is not established.
- What a daemon or provider does when the identity its `identity` names is not signed in, or is signed
  out — whether it refuses to start or starts and fails each handshake — was not traced. Either
  way the fix is a login under that root (and a restart if the program did not start).
- Whether a `remote` Postgres needs anything prepared in it before the daemon's first start was not checked.

## When this task is done

Load the `diverge` skill again before starting the next Diverge task: it is the index, and it says which
skill the next task needs.
