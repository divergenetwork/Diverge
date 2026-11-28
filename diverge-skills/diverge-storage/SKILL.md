---
name: diverge-storage
description: Storage in Diverge through the `diverge` CLI — volumes (`diverge run volumes create|get|list|edit|delete|stat|tag|untag|filetree|download|upload|transfer`; modes persistent/ephemeral/read_only, sizes in bytes and the provider's 4 MiB floor, stored vs fixed volumes, mounting a volume into an agent or tool as a volume mount or a FUSE mount, what the provider's volume capabilities mean, `held` and `in_use` answers), and postgres (`diverge run postgres get|list` and `diverge run postgres compartments list|get|tag|untag|delete`; the one database the daemon serves, the three compartments every container reaches by naming `instance`, `template` or `image`, which of them are shared, how each is removed, listing open container connections). Load it when you create or resize a volume, put files into or take files out of a volume, mount storage into a container, debug a start error about a volume, or need to know where a container's database is.
---

# Diverge storage: volumes, postgres

Two kinds of state outlive a container:

- A **volume** is a directory a *provider* keeps on its disk, mounted into containers. It belongs to
  the provider, is named by `{provider, name}`, and outlives every container and connection.
- **Postgres** is the one database the *daemon* serves into its containers. Each container reaches
  three compartments of it with no credential: its own, its template's and its image's.

Every command is `diverge run <family> <leaf> [BODY]`, the body one JSON object. `request-schema` and
`response-schema` beneath any leaf print the exact shapes and dial nothing. Global flags
(`--daemon`, `--account`, `--jq`, `--timeout`) go right after `run`. Every body also accepts `relay`
(see `diverge-daemons`). Every leaf can answer `"forbidden"` (no grant; see `diverge-access`) and
`{"error":…}`.

Two shapes recur:

```json
{"issuer": "https://identity.diverge.network", "subject": "ronald.provider"}                               // a provider's identity
{"provider": {"issuer": "https://identity.diverge.network", "subject": "ronald.provider"}, "name": "data"}  // a volume reference
```

Get a provider's identity from `diverge run providers list` (see `diverge-daemons`). Paths inside a
volume or a container are **arrays of components** (`["site","index.html"]`), never strings; no
component is empty, `.` or `..`, or holds `/`.

---

# Volumes

## What a volume is

- **Stored volume** — one a client created. On the provider it is an ext4 filesystem in a sparse
  image file; its size is *reserved*, not taken. It can be resized, re-moded and deleted.
- **Provided volume** — one the provider provides as it is, which no client made. A list shows it
  with `"provided": true` (the member is absent on any other volume). Its size and mode cannot be
  edited and it cannot be deleted: `volumes edit` and `volumes delete` answer `"provided"`, and
  asking again changes nothing. Everything else works as on any volume: mount it, FUSE serve it,
  read, write, stat and tag it. On our own provider these are the directories its configuration
  names (`diverge provider volumes fixed`); it may list one only to some identities.
- **Whether a provider creates volumes** is its capability, per mode: `create` in the mode's object
  (see "What the provider's capabilities mean"). A provider may keep volumes of a mode and create
  none; `volumes create` is then answered `unsupported` without the provider being asked. Our own
  provider creates volumes only if its configuration has `volumes.stores`.

**Mode** is a fact of the volume, set at create and changed only by an edit:

| mode | what a container's writes become | who may hold it at once (at most) |
|---|---|---|
| `persistent` | kept in the volume | any number of running containers sharing one filesystem, **or** one FUSE serve; never both |
| `ephemeral` | discarded with the container; every container starts from the volume as it is | any number of containers and FUSE serves |
| `read_only` | refused inside the container | any number of containers and FUSE serves |

Uploads at rest land in all three modes; the mode governs containers, not `volumes upload`.

**Size** is `bytes`. The provider refuses fewer than **4 MiB (4194304)**: less cannot hold the
filesystem's journal. A too-small size comes back as `{"error":…}`.

**At rest, or held.** A volume is read, written, walked, resized and deleted only *at rest*. While a
running container has it, or while a download, upload, transfer or FUSE serve is on it, those requests
answer `"held"`: wait and ask again. Agents are stopped after the daemon's `idle_seconds` (default 10) of
disuse, so a volume an idle agent mounted frees itself shortly.

## Create

When: before the first agent or tool that needs disk state.

```bash
diverge run volumes create '{
  "provider": {"issuer":"https://identity.diverge.network","subject":"ronald.provider"},
  "name": "data", "bytes": 1073741824, "mode": "persistent"}'
```

All four members are required. `name` is the handle forever (it cannot be renamed): one path
component, not starting with `.`, with no `/`, `\` or NUL, and not a fixed volume's name.

| answer | meaning |
|---|---|
| `"created"` | the volume exists and a list shows it |
| `"no_provider"` | no provider on record has that identity |
| `"exists"` | that provider already lists a volume of the name; nothing changed |
| `"insufficient_capacity"` | the provider cannot reserve that many bytes; ask smaller |
| `"unsupported"` | the provider keeps no volumes, or creates none of that mode (the mode's `create` is not among its capabilities); ask another provider |
| `{"error":…}` | could not ask (provider not connected) or the provider refused (bad name, under 4 MiB) |

Grant: `{"volumes":["create"]}`.

## Get, list, stat

```bash
diverge run volumes get '{"volume":{"provider":{…},"name":"data"}}'
diverge run volumes list                                   # every volume the grants reach
diverge run volumes list '{"modes":["persistent"],"mounted":false,"count":20}'
diverge run volumes list --watch '{"all_tags":["prod"]}'   # keep the stream open
diverge run volumes stat '{"volume":{"provider":{…},"name":"data"}}'
```

A volume as `get` (`{"found":{…}}`) and `list` (`{"added":{…}}`) report it:

```json
{"provider":{"issuer":"…","subject":"ronald.provider"},"name":"data","bytes":1073741824,"mode":"persistent",
 "created":"2026-11-10T09:00:00Z",
 "agents":[{"template":"…","index":0,"name":"worker"}],
 "tools":[{"record":{"template":"…","index":0,"name":"notes"}}],"tags":["prod"]}
```

`agents` and `tools` are the records that name the volume in their mounts, running or not (absent
when empty; a tool is `{"record":{…}}` for a standalone tool or `{"dependency":{…}}` for a dependency
tool). There is no creator: a volume is the provider's.

`list` filter members, all optional: `providers` (identities), `names`, `modes`, `mounted`
(true/false: whether any agent or tool names it in its mounts), `all_tags`, `any_tags`,
`created_from`, `created_to` (RFC 3339), and `count` (a cap). It prints one `{"added":…}` per volume,
provider by provider and by name, then `"listed"`, and ends; with `--watch` it stays open and prints
`{"added":…}`, `{"changed":…}`, `{"removed":…}` as things change. Only volumes of **connected**
providers appear: a provider on record but not connected contributes nothing.

`stat` walks the volume (so it needs the volume at rest) and answers what a listing leaves out:
`{"stat":{"bytes_used":123456,"dirhash":"h1:…"}}` — bytes in use and a content hash (Go's `dirhash`
`h1:` form; equal hashes mean equal content). Other answers: `"not_found"`, `"held"`.

Grants: `get`, `list`, `stat` actions over the volume.

## Edit (size, mode)

```bash
diverge run volumes edit '{"volume":{"provider":{…},"name":"data"},
  "patch":[{"op":"replace","path":"/bytes","value":2147483648}]}'
diverge run volumes edit '{"volume":{…},"patch":[{"op":"replace","path":"/mode","value":"read_only"}]}'
```

`patch` is a JSON Patch (RFC 6902) applied to the document `{"bytes":…,"mode":…}`; those two members
are the whole of what can change. Both in one patch are one edit: a size the provider refuses changes
the mode no more than the size. An empty patch, or one that changes nothing, is `"edited"` with
nothing asked.

| answer | meaning |
|---|---|
| `"edited"` | the volume is as the patch states |
| `"not_found"` | no such volume |
| `"insufficient_capacity"` | the provider cannot reserve that many bytes |
| `"content_too_large"` | the volume holds more than the smaller size asked |
| `"held"` | a running container has it, or a download/upload/transfer is on it |
| `"provided"` | the provider provides this volume as it is (`"provided": true` in a list): its size and mode are not edited, ever |
| `{"error":…}` | a patch that does not apply, under 4 MiB, a provider failure |

Changing the mode changes what every existing mount of the volume means: a mount states the mode it
expects, and a mismatch is a start error (see mounting). Grant: `edit`.

## Delete

```bash
diverge run volumes delete '{"volume":{"provider":{…},"name":"data"}}'
```

Irreversible: the content is gone. Answers: `"deleted"`, `"not_found"`, `"in_use"`, `"forbidden"`,
`{"error":…}`. **`in_use`** means some agent or tool of the daemon's names the volume in its mounts —
whether or not it is running — or a download, upload, transfer or FUSE serve is on it now. Find the mounters with
`volumes get` (`agents`, `tools`), take the volume out of their mounts (an edit of the agent or tool)
or delete them, and ask again. To get space back and keep the name, delete and create. Grant:
`delete`.

## Tag, untag

```bash
diverge run volumes tag   '{"volume":{…},"tags":["prod","team-a"]}'     # "tagged"
diverge run volumes untag '{"volume":{…},"tags":["team-a"]}'            # "untagged"
```

Tags are the daemon's, kept beside the provider's listing and dropped when the provider stops listing
the volume. Tagging a tag already there, or removing one not there, is not a failure. Tags are what
grants narrow by (`within`), so tag volumes you want to delegate. Answers also include `"not_found"`.
Grant: the tagging shape, `{"volumes":{"actions":["tag","untag"],"within":"any","tags":"any"}}`.

## Files on a volume: filetree, download, upload, transfer

All four need the volume **at rest** (`"held"` otherwise) and hold it for their length.

```bash
# what is in it: one snapshot of the tree (path [] is the whole volume)
diverge run volumes filetree '{"volume":{…},"path":[]}'
diverge run volumes filetree '{"volume":{…},"path":["site"]}'

# a file or a directory out of it, to a local path
diverge run volumes download --path ./index.html '{"volume":{…},"path":["site","index.html"]}'
diverge run volumes download --path ./backup     '{"volume":{…},"path":[]}'

# one local file in, at a path
diverge run volumes upload --path ./config.json '{"kind":"file","volume":{…},"path":["etc","config.json"]}'

# several files of a local directory in, under a path ([] is the volume's root)
diverge run volumes upload --path ./site '{"kind":"directory","volume":{…},"path":["site"],
  "files":["index.html","css/main.css"]}'

# copy from the volume into an agent, a tool, a dependency tool or a volume; bytes never reach you
diverge run volumes transfer '{"volume":{…},"path":["site"],"to":{"agent":{"name":"worker"},"path":["srv","site"]}}'
diverge run volumes transfer '{"volume":{…},"path":["site"],"to":{"volume":{"provider":{…},"name":"backup"},"path":["site"]}}'
```

- **filetree** answers `{"tree":[…]}`: nodes with `type` `file` (`name`, `size`, times), `directory`
  (`name`, `children`) or `symlink` (`name`, `path`). `"not_directory"` when a file is at the path;
  `"not_found"` when nothing is. It is a snapshot; nothing is watched.
- **download** `--path` is required: the local file a file download is written to, or the local
  directory a directory download is written under (parents made). It prints one
  `{"path":[…],"bytes":N}` per file written, or `"not_found"` / `"held"`. A directory with no file
  prints nothing.
- **upload** `--path` is required: the local file, or the local directory each name in `files` is
  read under. `files` are `/`-separated paths relative to that directory, at least one, none twice;
  **the CLI uploads only the files you list**, it does not walk the directory. Each file lands whole,
  parents made, replacing what was at its path and touching nothing else. Answers `"uploaded"`,
  `"not_found"`, `"held"`.
- **transfer** `to` is exactly one of `{"agent":…,"path":[…]}`, `{"tool":…,"path":[…]}`,
  `{"dependency":{"template":…,"owner":…},"path":[…]}`, `{"volume":…,"path":[…]}`; the destination
  `path` has at least one component. A file lands at `path`; a directory lands *as* `path`, nothing
  already there removed. An agent, a tool or a dependency tool that is not running is started for the copy and
  stopped after. Answers `"transferred"`, `"not_found"`,
  `"no_destination"`, `"held"` (either end).

Grants: `filetree`, `download`, `upload`, `transfer` over the source volume; a transfer's
destination is judged by its own grant (`upload` over a destination volume).

## Mounting a volume into an agent or a tool

Mounts are stated on the **create** of an agent or a tool (or a later edit), never on a template.
Naming a volume in a mount takes the `mount` grant over it. Two ways:

1. **Volume mount** — the container is pinned to the volume's provider and the volume is mounted
   directly:

   ```json
   {"template":"<id>","name":"worker",
    "provider":{"identity":{"issuer":"…","subject":"ronald.provider"},
      "volume_mounts":[{"volume_name":"data","volume_relative_path":[],
                        "volume_mode":"persistent","container_path":["data"]}]}}
   ```

2. **FUSE mount** — a file or a directory of *any* provider's volume, served live across the daemon:

   ```json
   {"template":"<id>","name":"worker",
    "fuse_directory_mounts":[{"provider":{"issuer":"…","subject":"ronald.provider"},"volume_name":"creds",
        "volume_relative_path":[],"volume_mode":"persistent","container_path":["root",".config","tool"]}],
    "fuse_file_mounts":[{"provider":{…},"volume_name":"creds","volume_relative_path":["token.json"],
        "volume_mode":"ephemeral","overlay_disk":16777216,"container_path":["etc","token.json"]}]}
   ```

Key rules: `volume_mode` must equal the volume's actual mode or the start fails; `overlay_disk` is
required for an `ephemeral` FUSE mount and forbidden otherwise; no mount path may be the root or lie
inside another mount; a persistent volume cannot be FUSE-served while a container mounts it or
another serve holds it; a FUSE mount with `"daemon":"<record id>"` reaches a volume on another daemon,
judged here only by `relay` over that record; a FUSE mount whose serve is lost ends the run
(`mount_lost`). Full rules, the capability flags and every start error are in
[references/mounting.md](references/mounting.md). The create and edit commands themselves are in
`diverge-agents` and `diverge-tools`.

---

# Postgres

The daemon serves exactly **one** database into its containers, and keeps its own records in it too.
Which database is the daemon's `postgres` setting (`diverge daemon postgres get`: `local`, a
Postgres the daemon runs beside itself, the default; or `remote`, one it dials), read at start.
Nothing on the wire changes it; see `diverge-config`.

What a container sees: its driver dials its own proxy at `127.0.0.1:81`, user `diverge`, **no
password**, and names one of three databases. The daemon authenticates toward the real database for
it. Each name is one **compartment**: a login role and a schema of the same name, search path pinned
to it. Unqualified table names land there, and another compartment's tables are `permission denied`.

```
postgres://diverge@127.0.0.1:81/instance?sslmode=disable
postgres://diverge@127.0.0.1:81/template?sslmode=disable
postgres://diverge@127.0.0.1:81/image?sslmode=disable
```

Which compartment each name is:

| container | `instance` | `template` | `image` |
|---|---|---|---|
| an agent | that agent's own | every agent's of the same agent template | every agent's of the same image digest |
| a standalone tool | that tool's own | every tool's of the same tool template | every tool's of the same image digest |
| a dependency tool, scope `instance` | that agent's, for that dependency template | every tool's of that dependency template, across all agents | every tool's of the same image digest |
| a dependency tool, scope `template` | **none**: the connection is refused | the same | the same |

- One connection reaches exactly one compartment. A program may hold any number of connections to
  each, so a pool per database is the ordinary case. No single query reaches two compartments.
- Any other database name, `diverge` included, is refused with Postgres's own error for a database
  that does not exist (SQLSTATE `3D000`). So is `instance` from a `template` dependency tool, which runs
  for no one agent.
- A compartment is on record from the moment its agent, tool or dependency tool is made. Its role and
  schema are made at the first connection that names it.
- A program built with the Rust SDK asks for the URL with
  `client.postgres_url(Database::Instance)` (or `Template`, `Image`).

How each compartment goes away:

| compartment | removed |
|---|---|
| an agent's `instance` | with the agent's delete |
| an agent's `template` | with the agent template's delete |
| a standalone tool's `instance` | with the tool's delete |
| a standalone tool's `template` | with the tool template's delete |
| a dependency tool's `instance` | with its agent's delete |
| a dependency template's `template` | only by `postgres compartments delete`, and only while no agent's latest registration declares that dependency template |
| an `image` | only by `postgres compartments delete`. An agent's or a standalone tool's image: only while no existing template contains the digest. A dependency tool's image: only while no agent's latest registration declares a dependency template of that digest |

Removing a compartment drops its role, its schema and every table in it. It cannot be undone.

## `postgres get`: which database

```bash
diverge run postgres get        # {"mode":{"kind":"local"}}  or  {"mode":{"kind":"remote","url":"postgres://user@host:5432/db"}}
```

The body is `{}`. A remote URL is answered with its password taken out. Grant: `{"postgres":["get"]}`.

## `postgres list`: open container connections

```bash
diverge run postgres list
diverge run postgres list --watch
```

One line per connection a container holds open through the database now, oldest first, then
`"listed"`:

```json
{"added":{"id":17,"container":{"agent":{"template":"…","index":0}},"opened":"2026-11-10T09:01:02Z"}}
```

`container` is `{"agent":…}`, `{"tool":…}` (each by template and index) or
`{"dependency":{"template":…,"owner":…}}`. `compartment` is which of the container's three the
connection reaches, in the form below. A container holding a pool is listed once per connection,
told apart by `id`. There is no filter and no count. With `--watch`, connections are printed as
`{"added":…}` when they open and `{"removed":…}` when they close. Use it to see which containers are
actually using their database. Grant: `{"postgres":["list"]}`.

## `postgres compartments`: list, get, tag, untag, delete

A compartment is named by one JSON object, its one member saying which of the three it is:

| form | names |
|---|---|
| `{"instance":{"agent":<agent>}}` | an agent's own. `<agent>` is `{"name":…}` or `{"template":…,"index":…}` |
| `{"instance":{"tool":<tool>}}` | a standalone tool's own, named the same two ways |
| `{"instance":{"dependency":{"template":"<dependency template id>","owner":{"agent":<agent>}}}}` | a dependency tool's own |
| `{"template":{"kind":"agent"\|"standalone_tool"\|"dependency_tool","template":"<id>"}}` | a template's |
| `{"image":{"digest":"sha256:…"}}` | an image's |

The daemon always answers an agent or a tool by template and index. A list filter and a grant's
`within` match that form only.

```bash
diverge run postgres compartments list                              # {"added":…}… "listed"
diverge run postgres compartments list '{"deletable":true}'         # what a delete would be allowed for now
diverge run postgres compartments list '{"databases":["image"]}' --watch
diverge run postgres compartments get '{"compartment":{"image":{"digest":"sha256:…"}}}'
diverge run postgres compartments tag '{"compartment":{"template":{"kind":"dependency_tool","template":"<id>"}},"tags":["keep"]}'
diverge run postgres compartments untag '{"compartment":{"instance":{"agent":{"name":"echo"}}},"tags":["keep"]}'
diverge run postgres compartments delete '{"compartment":{"image":{"digest":"sha256:…"}}}'
```

One list item:

```json
{"added":{"compartment":{"image":{"digest":"sha256:…"}},"kind":"agent","created":"2026-11-25T09:01:02Z","deletable":false,"connections":2}}
```

| member | meaning |
|---|---|
| `compartment` | which one, in the form above |
| `kind` | `"agent"`, `"standalone_tool"` or `"dependency_tool"`: what kind of container it is of. For an `image`, the `type` of the image's document |
| `tags` | absent when empty |
| `created` | since when it is on record |
| `deletable` | whether `delete` would be allowed now, connections apart. Always `false` for the compartments that go with an agent, a tool or their template |
| `connections` | how many connections containers hold through it now |

List filter, all optional; a list matches any one of its items:

| member | matches |
|---|---|
| `compartments` | exactly these, each in the answered form |
| `databases` | `"instance"`, `"template"`, `"image"` |
| `kinds` | `"agent"`, `"standalone_tool"`, `"dependency_tool"` |
| `templates` | template ids: a `template` compartment's own, an agent's or a tool's `instance`'s template, a dependency tool's `instance`'s dependency template |
| `images` | image digests: an `image` compartment's own |
| `deletable` | `true` or `false` |
| `all_tags`, `any_tags` | tags |
| `created_from`, `created_to` | inclusive RFC 3339 bounds |
| `count` | keep to the first N that match |

Answers:

| leaf | answers |
|---|---|
| `list` | `{"added":<item>}`…, `"listed"`; with `--watch` also `{"changed":…}`, `{"removed":…}` |
| `get` | `{"found":<item>}`, `"not_found"` |
| `tag` | `"tagged"`, `"not_found"` |
| `untag` | `"untagged"`, `"not_found"` |
| `delete` | `"deleted"`, `"not_found"`, `"not_deletable"`, `"in_use"` |

- `"not_deletable"`: the compartment goes with what it is of, or what keeps it is still there (a
  template that contains the image, an agent that declares the dependency template). Nothing changed.
- `"in_use"`: a container holds a connection through it, or a dependency tool of its template runs.
  Nothing changed. Wait for the container to stop and retry. A tool stays up until nothing holds it
  for `idle_seconds`, and an attachment holds an eager one (see `diverge-tools`).
- Deleting a dependency template's `template` compartment also removes **every tool of that
  template** (the template-scoped one, and any instance ones no agent declares any more, each with its own
  `instance` compartment) and the stored template. An agent that declares the template again at its
  next start gets a new tool with new, empty compartments. This is the only way a `template`
  dependency tool's record goes away.

Grants: `{"postgres_compartments":{"actions":["get","list","delete"],"within":…}}` and the tagging
shape, `{"postgres_compartments":{"actions":["tag","untag"],"within":"any","tags":"any"}}`.

---

## Grants at a glance

| family | make (bare array) | over actions (`{"actions":[…],"within":…}`) | tagging |
|---|---|---|---|
| `volumes` | `create` | `get`, `list`, `delete`, `edit`, `stat`, `download`, `upload`, `transfer`, `filetree`, `mount`, `fuse_serve` | `tag`, `untag` |
| `postgres` | — | bare array: `get`, `list` | — |
| `postgres_compartments` | — | `get`, `list`, `delete` | `tag`, `untag` |

`mount` (volumes) is not a command: it is what naming a volume in a mount takes. `fuse_serve` (volumes) is what **another daemon's account** holds here
so that its containers can FUSE-mount the volume; `diverge run volumes fuse-serve` exists only for its
schemas, and the CLI refuses to put the request. Admins hold everything. Shapes, `within` and diagnosing
`forbidden` are in `diverge-access`.

## Not verified

- No request was run live while writing this (the daemon was down); answers are from the SDK's
  response types, the daemon's handlers and the CLI's `response-schema`.
- The largest size a volume may have (a resize refuses sizes whose 4 KiB block count exceeds 32
  bits, about 16 TiB; whether create applies the same bound was not traced).

## When this task is done

Load the `diverge` skill again before starting the next Diverge task: it is the index, and it says which
skill the next task needs.
