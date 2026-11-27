# Dependency tools in detail

A dependency tool is a tool container the daemon deploys because an agent's program declared it when the
agent's container registered. Nothing on the daemon's wire creates one: you cause one by making an agent
whose image declares it, and messaging that agent.

## The dependency template

What the program declares, one object per dependency. For the agent-loop images in this repository it is
copied from the agent template's `arguments.dependencies`; a custom agent image may hard-code its own. Check
the agent image's `DIVERGE.md` (`diverge run images read --info`, `diverge-images`).

```json
{
  "image": {"digest": "sha256:…", "references": [{"registry": "docker.io", "name": "diverge-local/memory"}]},
  "extra_memory": 0,
  "extra_disk": 0,
  "arguments": {},
  "fuse_directory_mounts": [{"agent_path": ["workspace"], "tool_path": ["workspace"]}]
}
```

```json
{
  "image": {"digest": "sha256:…", "references": [{"registry": "docker.io", "name": "diverge-local/taskboard"}]},
  "extra_memory": 134217728,
  "extra_disk": 0,
  "arguments": {}
}
```

The first is the `memory` image, whose `DIVERGE.md` says `"mode":"lazy","scope":"instance"`, so it may name
a mount. The second is the `taskboard` image, whose document says `"mode":"eager","scope":"template"` and
`"memory":268435456`: with `"extra_memory":134217728` its container's memory limit is the sum, 402653184
bytes.

| member | meaning |
|---|---|
| `image` | digest and references, as in any template. References are not in the id. The image must ship a valid `DIVERGE.md` that the daemon can read, and that document's `type` must be `dependency_tool`; otherwise the registration fails and the agent does not start. |
| `extra_memory`, `extra_disk` | bytes, required: what the container gets beyond the `memory` and `disk` its image's document says it needs. The limit is the sum. `0` is what the image needs and no more. |
| `arguments` | handed to the tool's program verbatim; the tool image's `DIVERGE.md` gives the schema. A credential the tool needs is a member of them. |
| `fuse_file_mounts`, `fuse_directory_mounts` | optional. Allowed only where the tool image's document says `"mode":"lazy"` and `"scope":"instance"`; a declaration that names a mount of an image of any other mode or scope is refused. Each `{"agent_path":[…],"tool_path":[…]}`: a file or a directory of the **agent's** container served live into the tool's container. Nothing is copied; both containers see the same path while both run. Neither path may be the root. |

There is no name and no description, and there is no `mode` and no `scope`: the tool image's own
`DIVERGE.md` states those two (`"mode"`: `"lazy"`, runs while it is used, or `"eager"`, kept up while any
agent is attached; `"scope"`: `"instance"` or `"template"`, how far the tool's container is shared). Read them with `diverge run images read --info`. The **id** is the lowercase hex SHA-256 of the
template's canonical JSON: compact, keys sorted at every depth, absent members omitted, image references
dropped, the mounts included. Declaring the same template twice in one list is refused. A
dependency template cannot name the caller's volumes or a provider.

## Mode, scope, owner

| the tool image says | one tool per | `owner` in references and list items | container runs |
|---|---|---|---|
| `lazy` + `instance` | agent | `{"agent":{"template":…,"index":…}}` | from the agent's container start to its stop, every run |
| `lazy` + `template` | dependency template | `"template"` | from its first use until nothing has used it for `idle_seconds`; one container for every agent that declares it |
| `eager` + `instance` | agent | `{"agent":{…}}` | while that agent is attached (declares it) |
| `eager` + `template` | dependency template | `"template"` | while any agent at all is attached |

What a dependency tool keeps in Postgres does not follow its scope. Its program names a database, as
every container does (see `postgres` in `diverge-storage`):

| database | shared by | removed |
|---|---|---|
| `instance` | that one agent's tool of this dependency template. A `template` tool has none: the connection is refused | with the agent |
| `template` | every tool of this dependency template, across all agents | by `postgres compartments delete`, only while no agent declares the template |
| `image` | every tool of the same image digest | by `postgres compartments delete`, only while no agent declares a dependency template of that image |

- "Attached" for a dependency means: the agent's **last** registration declared the template. An agent is
  attached by registering (its container starting) with the declaration and detached by registering without
  it. The agent's container stopping does not detach it; an eager tool stays up between its agents' runs.
- An eager tool with no agent attached goes down after `idle_seconds`. An eager tool whose container ends is started
  again while any agent is attached, and at every daemon start.
- In a request you may name an agent owner either way (`{"agent":{"name":"echo"}}` works); answers always use
  the template-and-index form.
- The same eager tool template under the same owner is the same eager tool: a second agent declaring it joins it.

## The list item

`diverge run dependency-tools list '{"arguments":false}'` prints `{"added":<item>}` per tool then `"listed"`;
`get` prints `{"found":<item>}`. Both bodies must state `arguments`, `true` or `false`; a body without it is
refused. `true` needs the `arguments` grant over `dependency_tools` beside `get` / `list`: a `get` without it
answers `"forbidden"`, and a `list` answers `"forbidden"` unless the grant reaches every tool it would show.

| member | meaning |
|---|---|
| `id` | the dependency template's id. An agent calls its MCP tools as `<first 8 characters of id>_<tool>`. |
| `owner` | as above |
| `index` | its number among all dependency tools ever made from this template |
| `created`, `creator` | when the first declaring registration made it, and the agent that did |
| `tags` | the tool's own tags (not in the hash) |
| `template` | the template as the program declared it; its `arguments` only when the request said `"arguments": true` |
| `mode`, `scope` | what the tool image's document says: `"lazy"` or `"eager"`, and `"instance"` or `"template"`. Members of the item itself, not of its `template`. |
| `agents` | the agents attached now, each `{"agent":{template,index,name},"declared":"<time>"}`; absent when none |
| `active`, `last_active` | a call or read in flight; when that last changed |
| `deployed`, `last_deployed` | container up; when that last changed |
| `run` | `{"provider":{…},"id":"<container id>"}` while its container is up; absent otherwise |
| `permissions` | the grants its program declared at its last start; absent when none or never started |

## List filters

All optional; lists match any one of, except where noted.

| member | matches |
|---|---|
| `ids` | of any of these dependency template ids |
| `owners` | owned by any of these (`{"agent":…}`, `"template"`) |
| `creators` | first declared by any of these |
| `modes` | `"lazy"`, `"eager"` |
| `scopes` | `"instance"`, `"template"` |
| `depended` | `true`: some agent is attached; `false`: none |
| `agents` | agent names; the tool must be depended on by **every** one |
| `in_use` | `true`: its container runs now |
| `all_tags`, `any_tags` | tags |
| `created_from`, `created_to` | inclusive RFC 3339 bounds |
| `count` | keep to the first N that match |

From the agent's side, `diverge run agents list '{"dependency_tool_templates":["<id>"]}'` finds the agents
whose last registration declared a template, and each agent's item lists its `dependencies`.

## Deleting

There is no `dependency-tools delete`.

- An `instance` tool goes with its agent.
- Every tool of a dependency template, the `template` one included, goes when that template's `template`
  compartment is deleted:
  `diverge run postgres compartments delete '{"compartment":{"template":{"kind":"dependency_tool","template":"<dependency template id>"}}}'`.
  It answers `"not_deletable"` while any agent's latest registration declares the template, and
  `"in_use"` while any tool of it runs. See `diverge-storage`.
- After it, an agent that declares the template again at its next start gets a new tool with new,
  empty databases.

## Operating on a running dependency tool

`filetree`, `download`, `upload`, `transfer`, `exec`, `terminal`, `frontend` all take
`"dependency":{"template":…,"owner":…}` and act only on a container that is **already running**. They never
start one and never stop one; a tool that is down answers `"not_found"`. While `exec`, `terminal` or
`frontend` runs, the tool and the agent it was deployed for are held up.

To work on a lazy dependency: message the agent (which starts both), then run the command before the
agent goes idle, or keep the agent busy.

```bash
ID="$(diverge run dependency-tools list --jq '.added.id // empty' '{"arguments":false,"owners":[{"agent":{"name":"echo"}}]}' | tr -d '"' | head -1)"
diverge run dependency-tools exec --raw "{\"dependency\":{\"template\":\"$ID\",\"owner\":{\"agent\":{\"name\":\"echo\"}}},\"argv\":[\"ls\",\"/\"]}"
```

The log, by contrast, is readable at any time:

```bash
diverge run logs "{\"of\":{\"type\":\"dependency_tool\",\"template\":\"$ID\",\"owner\":{\"agent\":{\"name\":\"echo\"}}}}"
```
