---
name: diverge-config
description: Configuring and operating a Diverge installation. Load it to choose the config root (`--config`, `DIVERGE_CONFIG`, `~/.diverge`); to read or change any setting of the daemon (`diverge daemon port|idle_seconds|admins|identity|postgres …`), the provider (`diverge provider port|identity|incoming|outgoing|podman|containers|volumes|images|messages …`), the viewer (`diverge viewer port|redial|daemons|default …`) or the CLI itself (`diverge cli daemons …`, `diverge cli default …`); to move or wipe the provider's podman data, a volume store or the image Store; to sign an identity in, list or remove one (`diverge identities login`; the accounts at Diverge's own issuer, `diverge identity.diverge.network …`; `diverge run identities …` on the daemon's host); to start or stop the daemon, the provider or the viewer; or to work out what must be restarted after a change and why a connection or handshake is refused.
---

# Diverge: settings, identities, start and stop

Everything a Diverge program is told is a **setting**, and every setting is read and changed with a `diverge`
command. This skill covers every command that is not a request to a daemon's records — the settings of the
daemon, the provider, the viewer and the CLI, the identities signed in on this host, and starting and stopping
the servers — plus `diverge run identities`, the same identity work done on a daemon's host.

If you have not loaded the `diverge` skill in this task, load it first.

## The one rule

**Work through `diverge` commands only. Never open, list, read, search, copy or edit anything inside the config
root.** It holds the signed-in identities and their private keys, which are secrets. Everything you need to
read has a `get`; everything you need to change has a `set`, `add`, `delete`, `enable` or `disable`. If a
command cannot do what you need, say so to the user. Do not go around it.

## The config root

Every Diverge program on a host is told which **config root** to use. All of them choose it the same way:

| Order | The root is |
|---|---|
| 1 | the directory given as `--config <dir>` |
| 2 | else the directory named by the `DIVERGE_CONFIG` environment variable |
| 3 | else `~/.diverge` |

- `--config` is written before the command: `diverge --config <dir> daemon port get`.
- The root is created if it does not exist. A mistyped `--config` path silently gives a fresh installation with
  every setting at its default.
- A server started by the CLI runs on the root the CLI was given.
- One daemon and one provider per root. A second daemon on the same host is a second root, with its own
  settings and its own signed-in identities.
- Identities are per root: a program uses only the identities signed in under its own root.
- That is all there is to know about the root. What is inside it is not yours to look at.

## Ports

| Port | Who | Setting |
|---|---|---|
| 14979 | provider (all interfaces) | `diverge provider port` |
| 14980 | daemon (all interfaces) | `diverge daemon port` — and the address each CLI daemon entry and each viewer daemon entry dial |
| 14981 | viewer (this host only) | `diverge viewer port` |

## How settings work

Each setting is a command of its own under its program: `diverge daemon …`, `diverge provider …`,
`diverge viewer …`, `diverge cli …`. Every setting, with its default and meaning, is in
[references/settings.md](references/settings.md). What they share:

- **`get` reads, `set` writes.** A list has `get`, `add` and `delete`; `delete` takes the `index` that `get`
  prints, and the items after it move down one. Delete from the highest index down when deleting several.
- **A `get` prints the value in effect**, which is the default when the setting was never set. A number or a
  path is printed bare; an object or a list item as one JSON line.
- **A write prints nothing.** No output and exit 0 is success.
- **Nothing is given as JSON.** Values are plain arguments and flags.
- **A write to a daemon, provider or viewer setting is refused while that program is running**:
  `Error: the daemon is running on this root, as process <pid>, and nothing was written: stop it first, with`
  `` `diverge daemon stop` ``. Stop it, change the setting, start it. CLI settings have no such rule.
- **Only the value's shape is checked.** A wrong port, an identity that is not signed in, or a path that does
  not exist is found when the program starts.
- An optional part of the provider is switched with `enable` and `disable`, and asked with `enabled`. While it
  is disabled, a `get` of one of its settings prints `null` and a `set` is refused ("enable it first").
- The `issuer` string must be identical everywhere it appears: in logins, in an `identity`, in `admins`,
  `incoming`, CLI accounts and the viewer. `https://identity.diverge.network` and
  `https://identity.diverge.network/` are different issuers.

## The daemon's settings

```bash
diverge daemon port get                 # 14980
diverge daemon port set 15000
diverge daemon idle_seconds get         # 10
diverge daemon idle_seconds set 300
diverge daemon admins get               # {"index":0,"issuer":"https://identity.diverge.network","subject":"github.12345.admin"}   one line per administrator
diverge daemon admins add --issuer https://identity.diverge.network --subject github.12345.other
diverge daemon admins delete 0
diverge daemon identity get             # {"issuer":"…","subject":"…"}, or null
diverge daemon identity set --issuer https://identity.diverge.network --subject github.12345.daemon-a
diverge daemon postgres get             # {"kind":"local","max_connections":1024}  or  {"kind":"remote","url":"…","has_password":true}
diverge daemon postgres set local                             # the daemon's own Postgres, 1024 connections
diverge daemon postgres set local --max-connections 200
diverge daemon postgres set remote --url postgres://diverge@db.example.com:5432/diverge --password s3cret
```

- `identity` is who the daemon is: the one identity it presents on every connection. It must be signed in
  under this root. With none, the daemon completes no handshake. `identity set` takes both flags and replaces
  it whole; it cannot be cleared. It does not check that the identity is signed in.
- `admins` are admitted as clients whether or not they are accounts, and hold every grant. A fresh daemon has
  no account, so the first account is made by an administrator. `admins add` refuses one already listed.
- `idle_seconds`: a container unused this long is stopped; the next use starts it again.
- `postgres set` writes the whole choice and carries nothing over: `set local` without `--max-connections`
  writes 1024, and `set remote` without `--password` writes no password, whatever was set before. `--url` is
  required for `remote` and holds no password. `postgres get` never prints the password. A password given as a
  flag is in the shell's history. A different database is a different set of records.
- What the daemon announces itself as, whether it accepts other daemons and whether it hears messages are not
  settings here: they are on each provider's record (`diverge run providers edit`, skill `diverge-daemons`).

## The provider's settings

```bash
diverge provider port get               # 14979
diverge provider port set 15001
diverge provider identity get           # {"issuer":"…","subject":"…"}, or null
diverge provider identity set --issuer https://identity.diverge.network --subject github.12345.provider
diverge provider incoming get           # {"index":0,"issuer":"…","subject":"…"}   one line per identity that may dial in
diverge provider incoming add --issuer https://identity.diverge.network --subject github.12345.daemon-b
diverge provider incoming delete 0
diverge provider outgoing get           # {"index":0,"address":"…","issuer":"…","subject":"…"}   one line per peer the provider dials
diverge provider outgoing add --address relay.example.com:14979 --issuer https://identity.diverge.network --subject github.12345.relay
diverge provider outgoing delete 0
```

- `identity` is who the provider is; as the daemon's. With none, it completes no handshake.
- `incoming` is who may dial the provider. A peer whose verified identity is not listed is closed in the
  handshake; with none listed, nobody dials in. **A daemon that dials this provider must be here.**
- `outgoing` is for a provider that cannot be dialled: peers it dials itself and keeps redialling. `add` takes
  all three flags, and refuses only the same identity at the same address.

### podman

Read only by a provider that has containers or images enabled; one with neither runs no podman and needs none
installed.

```bash
diverge provider podman registries get                      # {"index":0,"host":"docker.io"}   one line per registry
diverge provider podman registries add registry.example.com:5000
diverge provider podman registries delete 2
diverge provider podman image_cache_disk get                # 34359738368   bytes
diverge provider podman image_cache_disk set 68719476736
diverge provider podman overlay_disk get                    # 34359738368   bytes
diverge provider podman overlay_disk set 68719476736
diverge provider podman storage_path get                    # the absolute path podman's data is kept at
diverge provider podman storage_path set /mnt/big/podman    # MOVES podman's data there, then changes the setting
diverge provider podman storage_path set /mnt/big/podman --no-transfer   # changes the setting, moves nothing
diverge provider podman storage_path delete                 # DELETES podman's data; the next start sets podman up again
```

- `registries`: hosts the provider pulls from when an image's references name them (default `registry.diverge.network`,
  `docker.io`, `ghcr.io`, `quay.io`). A host is what a reference names before its first `/`. The provider holds no registry
  credential, so only public images are pulled from one. Deleting every registry leaves none.
- `image_cache_disk`: the most podman's image cache may hold, in plain bytes. Lowering it deletes nothing at
  once; the provider trims the cache at its next container start.
- `overlay_disk`: one room for everything written temporarily, in plain bytes. A container's `disk`, an
  ephemeral volume mount's `overlay_disk` and an ephemeral FUSE serve's `overlay_disk` are each reserved
  from it, and one that does not fit is refused. It is read whether or not containers are enabled.
- `storage_path`: where podman keeps everything: on Linux its image store, on macOS and Windows the whole
  `diverge-provider` machine, which can be tens of gigabytes. By default it is inside the config root.
  - `set <path>` **moves the data**. Same drive or filesystem: a rename, instant, no extra space. Another
    drive: a copy, checked, with the old copy deleted afterwards; it needs room for all of it there.
  - The new path must not exist or must be empty, and must not be inside the old one or contain it.
  - All or nothing: on any failure everything is put back and the setting is unchanged. If it was interrupted
    (power loss, a kill), run the same command again.
  - The machine is stopped for you if it is running.
  - Linux: run it as root, as the provider runs. It is refused while anything is mounted there, or while
    podman's store there holds a container, pod or volume.
  - Windows: it needs a WSL with `wsl --manage --move` (2.3.11 or later; `wsl --update`).
  - `--no-transfer` leaves the old data where it is and starts podman empty at the new path. On Windows it is
    refused while the old machine exists; run `storage_path delete` first.
  - `delete` wipes podman's data at the current path so the next `provider start` sets it up from scratch.
    There is no undo and no prompt. Every image built or pulled into it must be built or pulled again. The
    image Store and the volumes are separate and untouched.

### containers

```bash
diverge provider containers enabled                         # true or false
diverge provider containers enable                          # 8 GiB memory
diverge provider containers disable
diverge provider containers memory get                      # 8589934592, or null while disabled
diverge provider containers memory set 17179869184
```

- A fresh provider runs no containers. With containers disabled every agent and tool run is refused as
  unsupported.
- `enable` on an already enabled provider changes nothing. `disable` forgets both numbers; a later `enable`
  starts from the defaults.
- The two numbers cap what all running containers may take together. `memory` also sizes the podman machine on
  macOS.
- Enabling containers makes the provider need podman at its next start.

### volumes

```bash
diverge provider volumes enabled
diverge provider volumes enable
diverge provider volumes disable
diverge provider volumes stores get                         # {"index":0,"path":"/mnt/volumes","capacity":107374182400}   one line each
diverge provider volumes stores capacity_used               # {"index":0,"capacity":…,"used":…,"free":…}   one line each
diverge provider volumes stores add --path /mnt/volumes --capacity 107374182400
diverge provider volumes stores edit 0 --capacity 214748364800
diverge provider volumes stores edit 0 --path /mnt/big/volumes    # MOVES the store and every volume in it
diverge provider volumes stores delete 0                    # DELETES the store and every volume in it
diverge provider volumes fixed get                          # {"index":0,"name":"datasets","path":"/srv/datasets","bytes":…,"mode":"read_only"}
diverge provider volumes fixed add --name datasets --path /srv/datasets --bytes 107374182400 --mode read_only
diverge provider volumes fixed delete 0                     # removes the entry, never the directory it names
```

- Two lists. `stores` are places new volumes are created. `fixed` are existing directories on the host that
  the provider offers as volumes as they are.
- The provider keeps volumes when either list has something. With both empty it keeps none, enabled or not: so
  `volumes enabled` only says the section is switched on.
- With a store, clients can create volumes of every mode. With fixed volumes only, clients see those and can
  create none. A fixed volume shows to clients as `"provided": true`, and they can neither edit nor delete it.
- A `get` prints nothing when there is none. `add`, `edit` and `delete` are refused while volumes are disabled.
- `stores add` refuses a path already in use by a store. `capacity` is the most the volumes in the store may
  reserve together, in plain bytes.
- `stores capacity_used`: `used` is what the volumes in the store *reserve* (the sum of their sizes), which is
  what counts against `capacity`; `free` is the rest. `stores edit --capacity` is refused below `used`.
- `stores edit --path` moves the store exactly as `podman storage_path set` moves podman's data (rename on one
  drive, checked copy across drives, all or nothing, run again to resume). `stores delete` has no undo and no
  prompt.
- `fixed add` takes `--name`, `--path` (an existing directory), `--bytes` (a declared size, not measured),
  `--mode` (`persistent`, `ephemeral` or `read_only`) and optionally `--authorize-hook`. It refuses a name
  already listed.

### images

```bash
diverge provider images enabled
diverge provider images enable                              # a Store of 32 GiB
diverge provider images disable                             # switches the Store off; its images are kept
diverge provider images path get                            # the absolute path, or null while disabled
diverge provider images path set /mnt/big/images            # MOVES the Store there
diverge provider images path set /mnt/big/images --no-transfer
diverge provider images path delete                         # DELETES every image in the Store
diverge provider images capacity get                        # 34359738368, or null while disabled
diverge provider images capacity set 214748364800
diverge provider images capacity_used                       # {"capacity":…,"used":…,"cached":…,"free":…}, or null while disabled
```

- The Store is where clients build and push images on the provider. Disabled, a build or push is refused.
- `enable` on an already enabled provider changes nothing. `disable` keeps the images; enabling again finds
  them.
- `path set` moves the Store as `podman storage_path set` does. `path delete` empties it, with no undo.
- `capacity_used`: `used` is what the images clients own take; `cached` is what is kept besides, which the
  provider removes whenever it needs room; `free` is `capacity` minus `used`. `capacity set` is refused below
  `used`.
- Enabling images makes the provider need podman at its next start.

### messages

```bash
diverge provider messages enabled
diverge provider messages enable                            # 30 messages per sending identity in every 60 seconds
diverge provider messages disable
diverge provider messages messages get                      # 30, or null while disabled
diverge provider messages messages set 120
diverge provider messages seconds get                       # 60, or null while disabled
diverge provider messages seconds set 30
```

- Messages are direct messages between the daemons connected to the provider. Disabled, none is relayed.
- `messages` is the count and `seconds` the window; each is 1 or more.

## The CLI's settings

The CLI keeps a list of daemons it may dial, each with a list of accounts to dial it as.

```bash
# the list of daemons
diverge cli daemons get                  # {"index":0,"name":"local"}   one line per daemon
diverge cli daemons add --name b --address ws://127.0.0.1:14982/
diverge cli daemons delete 1
diverge cli default get                  # {"index":0,"name":"local"}, or null
diverge cli default set 1                # by index

# one daemon: its index first, then the setting
diverge cli daemons 0 address get        # ws://127.0.0.1:14980/
diverge cli daemons 0 address set ws://10.0.0.5:14980/
diverge cli daemons 0 accounts get       # {"index":0,"name":"me","issuer":"…","subject":"…"}   one line per account
diverge cli daemons 0 accounts add --name me --issuer https://identity.diverge.network --subject github.12345.admin
diverge cli daemons 0 accounts delete 0
diverge cli daemons 0 default get        # {"index":0,"name":"me"}, or null: the account used when --account is not given
diverge cli daemons 0 default set 0      # by the account's index
diverge cli daemons 0 identity get       # null, or {"issuer":"…","subject":"…"}: the daemon's OWN identity
diverge cli daemons 0 identity set --issuer https://identity.diverge.network --subject github.12345.daemon-a
```

- A fresh CLI has one daemon, `local`, at `ws://127.0.0.1:14980/`, with no account.
- `daemons get` lists index and name only. What a daemon holds is read one setting at a time, after its index.
- `get`, `add` and `delete` work on the list and take no index before them; `address`, `accounts`, `default`
  and `identity` work on one daemon and need its index first.
- `daemons add` makes a daemon with no account, so `run` cannot use it until `accounts add`.
- An account is who the CLI dials **as**: an identity signed in under this root. `identity` is who the daemon
  must **be**; unset, whichever daemon answers at the address is accepted. `identity set` cannot clear it.
- `daemons delete` refuses the default daemon, and `accounts delete` refuses the daemon's default account: make
  another the default first.
- The names are the CLI's own labels (`--daemon`, `--account`). The daemon knows an account only by its
  identity.
- CLI settings take effect at the next command.

## The viewer's settings

The viewer keeps a list of daemons it may dial, each with accounts to dial it as, exactly as the CLI does, plus
two settings of its own. Every write is refused while the viewer runs on the root: stop it first.

```bash
diverge viewer port get                  # 14981: the loopback port the viewer hears its stop on
diverge viewer port set 14983
diverge viewer redial get                # 5, or never
diverge viewer redial set never

# the list of daemons, and the default
diverge viewer daemons get               # {"index":0,"name":"local"}   one line per daemon
diverge viewer daemons add --name b --address ws://127.0.0.1:14982/
diverge viewer daemons delete 1
diverge viewer default get               # {"index":0,"name":"local"}, or null
diverge viewer default set 1             # by index

# one daemon: its index first, then the setting
diverge viewer daemons 0 address get     # ws://127.0.0.1:14980/
diverge viewer daemons 0 address set ws://10.0.0.5:14980/
diverge viewer daemons 0 accounts get    # {"index":0,"name":"me","issuer":"…","subject":"…"}   one line per account
diverge viewer daemons 0 accounts add --name me --issuer https://identity.diverge.network --subject github.12345.admin
diverge viewer daemons 0 accounts delete 0
diverge viewer daemons 0 default get     # {"index":0,"name":"me"}, or null
diverge viewer daemons 0 default set 0   # by the account's index
diverge viewer daemons 0 identity get    # null, or {"issuer":"…","subject":"…"}: the daemon's OWN identity
diverge viewer daemons 0 identity set --issuer https://identity.diverge.network --subject github.12345.daemon-a
```

- A fresh viewer has one daemon, `local`, at `ws://127.0.0.1:14980/`, with no account: the same daemon a fresh
  CLI dials. The viewer's list is its own; adding a daemon or an account to the CLI adds nothing to the viewer.
- Everything said of the CLI's daemons above holds here: index before a daemon's setting, `delete` refusing the
  default, `identity set` that cannot be cleared.
- **The viewer does not start until the daemon it is to dial has an account** to dial it as (a default account,
  or one named at the start). Sign the identity in first, then `accounts add` and `default set`. What the viewer
  shows is what that account may see.
- `redial` is `never` or a whole number of seconds (1 or more) the viewer waits before dialling again.
- `port` is heard on this host only. Two viewers on one host are two roots, and need two ports.

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
| `accounts list` | The accounts signed in under this root. Asks nobody. |
| `accounts info <issuer> <id>` | What GitHub or Google says about that user right now. The `info` object is the issuer's own and differs between the two. |
| `subjects list <issuer> <id>` | The account's subjects that are signed in somewhere, with how many places each. |
| `refresh-tokens list <issuer> <id> <name>` | Where one subject is signed in: one entry per sign-in, with the address and program that started it and when it was last used. |
| `refresh-tokens end <issuer> <id> <name> <refresh-token-id>` | Signs one of them out. Tokens it already holds last until they expire (15 minutes). |

Facts and pitfalls:

- **`accounts login` needs a person once.** Read `user_code` from the printed line and give it to the person.
  If `browser_opened` is `false`, also give them `verification_uri`. `true` means a browser was started, not
  that anyone saw the page, so the address is always in the line.
- **`accounts login` prints no outcome.** Check with `accounts list`: the account appears once the code was
  approved. If it was denied or expired, nothing changes: run it again.
- The account's id is not chosen: it is whoever approved the code. One person signing in twice replaces the
  same account.
- An account whose issuer no longer accepts it answers an error that says it is signed out. Run
  `accounts login` again.
- A subject is listed only while it is signed in somewhere. Ending its last refresh token makes it vanish
  from `subjects list`; signing in under the same name brings the same subject back.
- No command here prints a token.
- `DIVERGE_IDENTITY_URL` points these commands at another copy of the service. Unset, they use
  `https://identity.diverge.network`.

## Identities on this host: `diverge identities`

An identity is one subject at one issuer, signed in once and kept signed in. All programs on a root share the
identities signed in under it. No command prints a token or a private key, and you must not look for either.

**Every identity has a private key on its host that never leaves it.** Each request for tokens proves the key
to the issuer (a DPoP proof, RFC 9449), the issuer binds the access token to it, and every connection made as
the identity is signed with it. A token copied off the wire or out of a log cannot be used by anyone else. The
key is made by itself, at the login or the first time a token is needed, and there is no command for it. The
same identity signed in on another host has a different key there.

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
| `--issuer` (required) | The issuer, exactly as it names itself. `https`, or `http` on loopback. It must offer a device authorization endpoint. |
| `--subject` (required) | The subject to sign in as. An approval as any other subject signs nothing in. |
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
  the subject named. If the code was denied or expired, or approved as another subject, nothing changes and
  nothing is reported: run `login` again.
- No browser is opened. Approve the printed code at the issuer.

At every issuer:

- A login replaces an identity already there, signed out or not. Logging in again is how a signed-out identity
  is repaired.
- A second `login` of the same identity while one is still waiting waits its turn behind the first.
- The issuer must be reachable.
- An identity whose sign-in was ended, or went unused past the issuer's lifetime for it (30 days at
  `identity.diverge.network`), is *signed out*: it is still listed, handshakes fail, and a new login fixes it.
- **The issuer must support DPoP with ES256.** `identity.diverge.network` does. A login at an issuer that does
  not list `ES256` under `dpop_signing_alg_values_supported` fails saying so, before anything is asked of it.
  An issuer that lists it but answers access tokens that are opaque, or not bound to the key, fails later with
  "answered an access token that is not bound to the identity's key".

### list, remove

```bash
diverge identities list
# {"issuer":"https://identity.diverge.network","subject":"github.12345.admin"}      one line per identity; signed-out ones included
diverge identities remove --issuer https://identity.diverge.network --subject github.12345.old
# {"removed":{"issuer":"…","subject":"github.12345.old"}}
```

`remove` removes the identity's key with it.

Removing the identity a running daemon or provider presents (its `identity`) breaks every later handshake of
that program. Removing one that a CLI account or the viewer names breaks that account.

## Identities on the daemon's host: `diverge run identities`

The same identities, on the host of the daemon you dial — for when the daemon runs somewhere you have no shell.
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
- Where the login could not begin, either way: `"held"` (another login of that identity is under way; try
  again), `"forbidden"`, or `{"error":…}` (issuer unreachable, or it has no device authorization endpoint).
- Signing an identity in does not make the daemon present it: which identity the daemon presents is its
  `identity` setting, read at start.

## Start and stop: `diverge daemon|provider|viewer start|stop`

```bash
diverge provider start > provider-start.out 2>&1; cat provider-start.out   # started provider, process <pid>
diverge daemon start   > daemon-start.out   2>&1; cat daemon-start.out     # started daemon, process <pid>

diverge viewer start   > viewer-start.out   2>&1; cat viewer-start.out     # started viewer, process <pid>
diverge viewer start --daemon b --account me > viewer-start.out 2>&1; cat viewer-start.out

diverge daemon stop      # stopped daemon      (or: daemon is not running — exit 0)
diverge provider stop
diverge viewer stop
```

**Never pipe a `start`** (no `|`, no `$(…)`): the detached server inherits the pipe and the shell hangs.
Redirect the command's output to a file of your own in your working directory, as above.

`start`:

- Starts the program detached, on the CLI's root, and returns once it is up.
- Refuses when one already runs on this root: `the daemon is already running on this root, as process <pid>`.
- If the program dies first, the error carries everything it said. Usual causes: the port is taken, a setting
  it refuses, and for a provider with containers or images enabled podman or `ssh` missing from `PATH`.
- A program that starts well prints nothing more.
- The first start on Windows or macOS of a provider with containers or images enabled creates and boots a
  podman machine and can take minutes.
- On Windows and macOS that machine (`diverge-provider`) runs only while the provider does: a start starts it,
  and `diverge provider stop` stops it after the provider's containers are removed, which adds some seconds.

`stop`:

- Asks the program to stop over its own port from this host, and returns once the whole teardown is done. A
  daemon stop ends every container run and its Postgres; a **provider stop removes every container it ran**.
- `daemon stop` dials **the CLI's default daemon** (`diverge cli default get`, then that daemon's `address`),
  not the daemon's own `port` setting. That address must be on this host (`127.0.0.1`, `localhost`, `::1`),
  else `… is not this host`. So each root's default CLI daemon must point at that root's own daemon, and its
  address must be updated when `diverge daemon port` changes. `provider stop` uses the provider's own `port`.
- `nothing is listening on port …: the program is not running, or is still starting` means it is still
  starting (or the CLI's default daemon names the wrong port). Wait and retry, or fix the address.
- A stop is honoured only from the same host; there is no remote stop.

The viewer is a desktop app, started and stopped the same way:

- `viewer start` opens the window on the viewer's default daemon as that daemon's default account.
  `--daemon <name>` and `--account <name>` pick others, by the names `diverge viewer daemons get` and
  `diverge viewer daemons <INDEX> accounts get` print. The window's title names the daemon it is on.
- One viewer runs per root. A second start is refused, as a daemon's is. To look at another daemon, stop the
  viewer and start it with `--daemon`.
- A start fails, with what the viewer said, when the daemon has no account to dial as, a name is not one the
  viewer has, or its port is taken.
- `viewer stop` closes the window. A person closing the window ends it too, and `viewer stop` then says
  `viewer is not running`.
- It needs a display: on a host with none, do not start it.

Order. Start: provider, then daemon (the daemon connects to the provider; a daemon redials its providers every
5 seconds, so a late provider is picked up). Stop in reverse: daemon, then provider. With several roots, stop
every daemon before the provider they share.

There is no `restart`: stop, then start.

## Updating the programs: `diverge update`

```bash
diverge update                      # one version up from the version installed
diverge update --version 1.2.0      # that version, newer or older
diverge update --os linux --arch aarch64   # for another platform than this host's
```

- With no `--version` it installs the version next after the one installed, never further in one run.
  Run it again to go up another. When the installed version is the newest it prints
  `<version> is the newest version` and changes nothing.
- It replaces every Diverge program on the host, the CLI included, and prints
  `updated to <version> for <os>/<arch>`.
- **Stop the daemon, the provider and the viewer first.** While any of them runs the update is refused
  with the same "is running" error a setting gives, before anything is downloaded.
- It is safe to interrupt. Until the programs are replaced nothing installed has changed; the replacing
  itself cannot be interrupted part way, and a failure in it puts the old programs back.
- `--os` is `linux`, `macos` or `windows`; `--arch` is `x86_64` or `aarch64`. A version that does not
  exist, or was not built for the platform, is an error and changes nothing.
- Start the programs again afterwards (`diverge provider start`, `diverge daemon start`).

## What needs a restart

No server notices a changed setting while it runs, and its settings cannot be changed while it runs anyway.

| Change | Takes effect |
|---|---|
| Any daemon setting | `diverge daemon stop`, change it, `diverge daemon start`. Agents and tools keep their records; running containers are ended and start again on next use. |
| `diverge daemon port` | As above, **and** update every address that names it: each CLI daemon entry and each viewer daemon entry. Change the CLI's address after the stop, since the stop dials the old one. Other daemons and providers that dial this one by address need their records edited. |
| `diverge daemon postgres` | As above. A different database is a different set of records. |
| Any provider setting | `diverge provider stop`, change it, `diverge provider start`. Stopping the provider removes every container it ran; connected daemons show it `disconnected`, redial every 5 seconds, and start containers again on next use. |
| A CLI setting | The next CLI command. Nothing to restart. |
| A viewer setting | `diverge viewer stop`, change it, `diverge viewer start`. |
| `diverge identities login` of an identity a running program already uses | No restart. |
| Records in the daemon (providers, accounts, roles, agents, …) | Immediate; those are `diverge run` requests, not settings. |

Before stopping anything, say what it will interrupt and make sure the user asked for it.

## Recipes

**From nothing to a working daemon you can talk to**, on one host, with Diverge's own issuer:

```bash
# 1. sign in: the account once (a person approves the code), then the identities
diverge identity.diverge.network accounts login github
diverge identities login --issuer https://identity.diverge.network --subject github.12345.admin    --client-id diverge
diverge identities login --issuer https://identity.diverge.network --subject github.12345.daemon   --client-id diverge
diverge identities login --issuer https://identity.diverge.network --subject github.12345.provider --client-id diverge

# 2. the provider: who it is, who may dial it, and that it runs containers
diverge provider identity set --issuer https://identity.diverge.network --subject github.12345.provider
diverge provider incoming add --issuer https://identity.diverge.network --subject github.12345.daemon
diverge provider containers enable

# 3. the daemon: who it is, and its first administrator
diverge daemon identity set --issuer https://identity.diverge.network --subject github.12345.daemon
diverge daemon admins add   --issuer https://identity.diverge.network --subject github.12345.admin

# 4. the CLI: dial the local daemon as the administrator
diverge cli daemons 0 accounts add --name me --issuer https://identity.diverge.network --subject github.12345.admin
diverge cli daemons 0 default set 0

# 5. start, and check
diverge provider start > provider-start.out 2>&1; cat provider-start.out
diverge daemon start   > daemon-start.out   2>&1; cat daemon-start.out
diverge run info        # {"info":{"version":"…"}}
```

Then add the provider to the daemon with `diverge run providers add` (skill `diverge-daemons`), and point the
viewer at the daemon by giving its daemon `local` the same account:
`diverge viewer daemons 0 accounts add --name me --issuer … --subject …`, `diverge viewer daemons 0 default set 0`,
then `diverge viewer start`.

**Add a second daemon to the CLI.** `diverge cli daemons add --name b --address ws://127.0.0.1:14982/`, find
its index with `diverge cli daemons get`, give it an account and a default, then `diverge run --daemon b info`.

**Let a daemon use a provider.** On the provider's root, with the provider stopped:
`diverge provider incoming add` with the daemon's identity, then start it. On the daemon's side nothing is a
setting: the provider record is added with `diverge run providers add` (skill `diverge-daemons`).

**Keep containers up longer.** `diverge daemon stop`, `diverge daemon idle_seconds set 300`, `diverge daemon start`.

**Give the provider volumes.** With the provider stopped: `diverge provider volumes enable`, then
`diverge provider volumes stores add --path <an absolute path> --capacity 107374182400`, then start it.
(Volumes themselves: skill `diverge-storage`.)

**Let the provider build and hold images.** With the provider stopped: `diverge provider images enable`, then
start it. (Images: skill `diverge-images`.)

## When something refuses

| Message | Cause | Fix |
|---|---|---|
| `the daemon is running on this root … stop it first` (or `the provider …`, `the viewer …`) | A setting was written while the program runs | Stop it, change the setting, start it |
| `` `containers` is disabled … enable it first `` (or `volumes`, `images`, `messages`) | A setting of a part that is switched off | `diverge provider <part> enable` |
| `no account was named, and the daemon … has no default` | The CLI's daemon has no default account (this stops even `request-schema`) | `diverge cli daemons <INDEX> accounts add …` and `default set`, or pass `--account` |
| `no daemon is named …` / `the daemon … has no account named …` | `--daemon` / `--account` name nothing the CLI has | `diverge cli daemons get`, then `diverge cli daemons <INDEX> accounts get` |
| `the daemon could not be dialled: …` | Daemon down, or wrong address in the CLI's daemon entry | `diverge daemon start`, or `diverge cli daemons <INDEX> address set …` |
| `the handshake did not complete: …` | The account's identity is not signed in under this root, is signed out, the issuer is unreachable, the daemon has no `identity` or its identity is not signed in under the daemon's root, or the CLI's daemon entry states an `identity` that is not the one that answered | `diverge identities list`; log in again; check `diverge daemon identity get` and `diverge cli daemons <INDEX> identity get` |
| `the daemon closed the connection without answering: the account may be none of the daemon's` | The identity is neither an administrator nor an account on that daemon | `diverge daemon admins add …` (daemon stopped), or have an admin create the account |
| A provider record stays `disconnected` | The daemon's identity is missing from the provider's `incoming`; the daemon or the provider has no `identity`, or it is not signed in; the provider's `identity` is not the one the record names; the provider is down | Fix the setting, restart the provider; the daemon redials by itself |
| A setting command fails saying a setting it did not touch is invalid | The installation's settings are damaged | Tell the user. Do not try to repair it yourself |
| The wrong installation is being used | `DIVERGE_CONFIG` is set | Pass `--config <dir>` explicitly |

## Not verified

- No setting command was run while writing this; the behaviour is from the code.
- On Linux the provider must run as root while the CLI and daemon usually do not; whether they should share
  one root is not established.
- What a daemon or provider does when its `identity` is not signed in, or is signed out — whether it refuses
  to start or starts and fails each handshake — was not traced. Either way the fix is a login under that root.
- Whether a `remote` Postgres needs anything prepared in it before the daemon's first start was not checked.

## When this task is done

Load the `diverge` skill again before starting the next Diverge task: it is the index, and it says which
skill the next task needs.
