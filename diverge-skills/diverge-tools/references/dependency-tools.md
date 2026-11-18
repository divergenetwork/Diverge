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
  "memory": 268435456,
  "disk": 268435456,
  "arguments": {},
  "secrets": ["SOME_API_KEY"],
  "mode": "lazy",
  "scope": "instance",
  "fuse_directory_mounts": [{"agent_path": ["workspace"], "tool_path": ["workspace"]}]
}
```

```json
{
  "image": {"digest": "sha256:…", "references": [{"registry": "docker.io", "name": "diverge-local/taskboard"}]},
  "memory": 268435456,
  "disk": 268435456,
  "arguments": {},
  "mode": "eager",
  "scope": "template"
}
```

| member | meaning |
|---|---|
| `image` | digest and references, as in any template. References are not in the id. The image must ship a valid `DIVERGE.md` that the daemon can read; otherwise the registration fails and the agent does not start. |
| `memory`, `disk` | bytes |
| `arguments` | handed to the tool's program verbatim; the tool image's `DIVERGE.md` gives the schema |
| `secrets` | names only. Every name must also be declared by the **agent's** template; the value comes from the vault key the agent binds that name to. Otherwise the registration fails. |
| `mode` | `"lazy"` (runs while it is used) or `"eager"` (kept up while any agent is attached). The members of the chosen mode sit flat beside it. |
| `scope` | required in both modes: `"instance"`, `"template"` or `"global"`. How far the tool, and its database, is shared. |
| `fuse_file_mounts`, `fuse_directory_mounts` | lazy with scope `instance` only; a lazy tool of another scope that names a mount is refused. Each `{"agent_path":[…],"tool_path":[…]}`: a file or a directory of the **agent's** container served live into the tool's container. Nothing is copied; both containers see the same path while both run. Neither path may be the root. |

There is no name and no description. The **id** is the lowercase hex SHA-256 of the template's canonical
JSON: compact, keys sorted at every depth, absent members omitted, image references dropped, secret names
sorted, `mode` and its members included. Declaring the same template twice in one list is refused. A
dependency template cannot name the caller's volumes or a provider.

## Mode, scope, owner

| declaration | one tool per | `owner` in references and list items | container runs | database lives as long as |
|---|---|---|---|---|
| `lazy` + `instance` | agent | `{"agent":{"template":…,"index":…}}` | from the agent's container start to its stop, every run | the agent |
| `lazy` + `template` | agent template | `{"agent_template":{"template":"<agent template id>"}}` | from its first use until nothing has used it for `idle_seconds`; one container for every agent of that template | the agent template |
| `lazy` + `global` | dependency template | `"global"` | the same, one container for every agent that declares it | until `dependency-tools delete` |
| `eager` + `instance` | agent | `{"agent":{…}}` | while that agent is attached (declares it) | the agent |
| `eager` + `template` | agent template | `{"agent_template":{"template":"<agent template id>"}}` | while any agent of that template is attached | the agent template |
| `eager` + `global` | dependency template | `"global"` | while any agent at all is attached | until `dependency-tools delete` |

- "Attached" for a dependency means: the agent's **last** registration declared the template. An agent is
  attached by registering (its container starting) with the declaration and detached by registering without
  it. The agent's container stopping does not detach it; an eager tool stays up between its agents' runs.
- An eager tool with no agent attached goes down after `idle_seconds`. An eager tool whose container ends is started
  again while any agent is attached, and at every daemon start.
- In a request you may name an agent owner either way (`{"agent":{"name":"echo"}}` works); answers always use
  the template-and-index form.
- The same eager tool template under the same owner is the same eager tool: a second agent declaring it joins it.

## The list item

`diverge run dependency-tools list` prints `{"added":<item>}` per tool then `"listed"`; `get` prints
`{"found":<item>}`.

| member | meaning |
|---|---|
| `id` | the dependency template's id. An agent calls its MCP tools as `<first 8 characters of id>_<tool>`. |
| `owner` | as above |
| `index` | its number among all dependency tools ever made from this template |
| `created`, `creator` | when the first declaring registration made it, and the agent that did |
| `tags` | the tool's own tags (not in the hash) |
| `template` | the template as the program declared it |
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
| `owners` | owned by any of these (`{"agent":…}`, `{"agent_template":…}`, `"global"`) |
| `creators` | first declared by any of these |
| `modes` | `"lazy"`, `"eager"` |
| `scopes` | `"instance"`, `"template"`, `"global"` |
| `depended` | `true`: some agent is attached; `false`: none |
| `agents` | agent names; the tool must be depended on by **every** one |
| `in_use` | `true`: its container runs now (what makes `delete` answer `"in_use"`) |
| `all_tags`, `any_tags` | tags |
| `created_from`, `created_to` | inclusive RFC 3339 bounds |
| `count` | keep to the first N that match |

From the agent's side, `diverge run agents list '{"dependency_tool_templates":["<id>"]}'` finds the agents
whose last registration declared a template, and each agent's item lists its `dependencies`.

## Deleting

`diverge run dependency-tools delete '{"id":"<dependency template id>"}'`

- Removes every tool of that template under every owner, each tool's database and contents, and the
  template record. Irreversible.
- `"in_use"` while any of those tools runs (a lazy one inside an agent's run, or an eager tool that is up).
  Nothing is deleted. Wait until the agents are undeployed and eager tools have retired, then retry. A global
  eager tool stays up as long as any agent is attached, so the agents that declare it must be deleted, or
  re-registered without it, before the eager tool can go down.
- `"not_found"`: no tool of the caller's has that template.
- Agents that declared it depend on it no more; one that declares it again at its next start gets a new
  tool with a new, empty database.

## Operating on a running dependency tool

`filetree`, `download`, `upload`, `transfer`, `exec`, `terminal`, `frontend` all take
`"dependency":{"template":…,"owner":…}` and act only on a container that is **already running**. They never
start one and never stop one; a tool that is down answers `"not_found"`. While `exec`, `terminal` or
`frontend` runs, the tool and the agent it was deployed for are held up.

To work on a lazy dependency: message the agent (which starts both), then run the command before the
agent goes idle, or keep the agent busy.

```bash
ID="$(diverge run dependency-tools list --jq '.added.id // empty' '{"owners":[{"agent":{"name":"echo"}}]}' | tr -d '"' | head -1)"
diverge run dependency-tools exec --raw "{\"dependency\":{\"template\":\"$ID\",\"owner\":{\"agent\":{\"name\":\"echo\"}}},\"argv\":[\"ls\",\"/\"]}"
```

The log, by contrast, is readable at any time:

```bash
diverge run logs "{\"of\":{\"type\":\"dependency_tool\",\"template\":\"$ID\",\"owner\":{\"agent\":{\"name\":\"echo\"}}}}"
```
