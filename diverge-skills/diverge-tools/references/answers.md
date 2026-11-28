# `standalone-tools` and `dependency-tools`: every leaf, its body, its answers

Every leaf has `request-schema` and `response-schema` beneath it and takes the global `run` options
`--daemon`, `--account`, `--jq`, `--timeout`. Every body may carry `relay`. Every leaf can also answer
`"forbidden"` (no grant reaches what is named; nothing changed) and `{"error":…}`; they are left out of the
tables. Bare answers are JSON strings; answers with a payload are one-member objects.

`<tool>` is `{"name":…}` or `{"template":…,"index":…}`. `<agent>` is the same two forms over agents.
`<dep>` is `{"template":"<dependency template id>","owner":<owner>}` with owner `{"agent":<agent>}`
or `"template"`.

## `diverge run standalone-tools templates`

| leaf | body | answers |
|---|---|---|
| `create` | `type:"tool"`, `image`, `extra_memory`, `extra_disk`, `arguments` | `{"created":"<id>"}`, `{"exists":"<id>"}` |
| `get` | `{"id","arguments":<bool>}` (required; `true` needs the `arguments` grant, else `"forbidden"`) | `{"found":{id, created, creator, tags, template}}`, `"not_found"` |
| `list` (`--watch`) | `arguments` (required bool; `true` is `"forbidden"` unless the grant reaches every template shown), `ids`, `creators`, `in_use`, `all_tags`, `any_tags`, `created_from`, `created_to`, `count` | `{"added":…}`…, `"listed"`; with `--watch` also `{"changed":…}`, `{"removed":…}` |
| `delete` | `{"id"}` | `"deleted"`, `"not_found"`, `"in_use"` |
| `tag` | `{"id","tags":[…]}` | `"tagged"`, `"not_found"` |
| `untag` | `{"id","tags":[…]}` | `"untagged"`, `"not_found"` |

## `diverge run standalone-tools`

| leaf | flags | body | answers |
|---|---|---|---|
| `create` | | `template`, optional `name`, `provider`, `fuse_file_mounts`, `fuse_directory_mounts` | `"created"`, `"in_use"`; `{"error":{"kind":"image",…}}` when the template's image has no valid `DIVERGE.md` the daemon can find; `{"error":…}` also when that document's `type` is not `standalone_tool` |
| `get` | | `{"tool":<tool>}` | `{"found":<item>}`, `"not_found"` |
| `list` | `--watch` | filter + `count` | `{"added":<item>}`…, `"listed"`; with `--watch` also `{"changed"}`, `{"removed"}` |
| `edit` | | `{"tool":<tool>,"patch":[…]}` | `"edited"`, `"not_found"`, `"active"`, `"in_use"` |
| `delete` | | `{"tool":<tool>}` | `"deleted"`, `"not_found"`, `"attached"` |
| `tag` | | `{"tool":<tool>,"tags":[…]}` | `"tagged"`, `"not_found"` |
| `untag` | | `{"tool":<tool>,"tags":[…]}` | `"untagged"`, `"not_found"` |
| `attach-agent-instance` | | `{"tool":<tool>,"agent":<agent>}` | `"attached"`, `"no_tool"`, `"no_agent"` |
| `detach-agent-instance` | | `{"tool":<tool>,"agent":<agent>}` | `"detached"`, `"no_tool"`, `"no_agent"` |
| `attach-agent-template` | | `{"tool":<tool>,"template":"<agent template id>"}` | `"attached"`, `"no_tool"`, `"no_template"` |
| `detach-agent-template` | | `{"tool":<tool>,"template":"<agent template id>"}` | `"detached"`, `"no_tool"`, `"no_template"` |
| `connect list-tools` | `--cursor` | `{"tool":<tool>}` | the MCP result; or `"not_found"` |
| `connect call-tool` | `--name` (required), `--arguments <JSON object>` | `{"tool":<tool>}` | the MCP result; or `"not_found"` |
| `connect list-resources` | `--cursor` | `{"tool":<tool>}` | the MCP result; or `"not_found"` |
| `connect read-resource` | `--uri` (required) | `{"tool":<tool>}` | the MCP result; or `"not_found"` |
| `filetree` | `--watch` | `{"tool":<tool>}` | `{"filetree":{"type":"snapshot",…}}`, changes with `--watch`; `"not_found"` |
| `download` | `--path <LOCAL>` (required) | `{"tool":<tool>,"path":[…]}` | one line per file written; `"not_found"` |
| `upload` | `--path <LOCAL>` (required) | `{"kind":"file","tool":<tool>,"path":[…]}` or `{"kind":"directory","tool":<tool>,"path":[…],"files":[…]}` | `"uploaded"`, `"not_found"` |
| `transfer` | | `{"tool":<tool>,"path":[…],"to":<destination>}` | `"transferred"`, `"not_found"`, `"no_destination"`, `"held"` |
| `exec` | `--raw` | `{"tool":<tool>,"argv":[…],"cwd":[…]?,"env":{…}?}` | `{"stdout":"<base64>"}`, `{"stderr":"<base64>"}`…, then `{"exited":<code>}` or `{"signaled":<n>}`; `"not_found"` |
| `terminal` | | `{"tool":<tool>,"cols":<n>,"rows":<n>}` | shell output raw on stdout; other answers as a JSON line on stderr |
| `frontend` | | `{"tool":<tool>,"method":…,"path":"/frontend/…","headers":[[name,value]…]}`, body on stdin | head as a JSON line on stderr, body raw on stdout; `"not_found"`, `"no_frontend"` on stderr |

A standalone tool's list item: `name`, `template`, `index`, `mode` (what the tool's image says), `provider` (where it runs or last ran),
`id` (container id while running), `creator`, `created`, `active`, `last_active`, `deployed`,
`last_deployed`, `attached_agent_instances` (each `{template,index,name}`), `attached_agent_templates`
(ids), `permissions`, `tags`.

List filter: `names`, `templates`, `creators`, `modes`, `active` (bool), `deployed` (bool),
`attached_agent_instances` (agent names; attached to every one), `attached_agent_templates` (ids; attached
to every one), `all_tags`, `any_tags`, `created_from`, `created_to`; plus `count`.

Transfer `<destination>`: `{"agent":<agent>,"path":[…]}`, `{"tool":<tool>,"path":[…]}`,
`{"dependency":<dep>,"path":[…]}`, `{"volume":{"provider":{"issuer","subject"},"name":…},"path":[…]}`. The
path has at least one component. A file source lands at the path; a directory source lands as the path, and
files already there that the source lacks stay.

## `diverge run dependency-tools`

| leaf | flags | body | answers |
|---|---|---|---|
| `get` | | `{"dependency":<dep>}` | `{"found":<item>}`, `"not_found"` |
| `list` | `--watch` | filter + `count` | `{"added":<item>}`…, `"listed"`; with `--watch` also `{"changed"}`, `{"removed"}` |
| `tag` | | `{"dependency":<dep>,"tags":[…]}` | `"tagged"`, `"not_found"` |
| `untag` | | `{"dependency":<dep>,"tags":[…]}` | `"untagged"`, `"not_found"` |
| `filetree` | `--watch` | `{"dependency":<dep>}` | as standalone |
| `download` | `--path <LOCAL>` (required) | `{"dependency":<dep>,"path":[…]}` | as standalone |
| `upload` | `--path <LOCAL>` (required) | `{"kind":"file"|"directory","dependency":<dep>,"path":[…],…}` | `"uploaded"`, `"not_found"` |
| `transfer` | | `{"dependency":<dep>,"path":[…],"to":<destination>}` | `"transferred"`, `"not_found"`, `"no_destination"`, `"held"` |
| `exec` | `--raw` | `{"dependency":<dep>,"argv":[…],…}` | as standalone |
| `terminal` | | `{"dependency":<dep>,"cols":<n>,"rows":<n>}` | as standalone |
| `frontend` | | `{"dependency":<dep>,"method":…,"path":…,"headers":[…]}` | as standalone |

A dependency tool's list item carries `mode` and `scope` as its own members (what the tool's image says),
beside its `template`. List filter members `modes` and `scopes` match them.

There is no `create`, `edit`, `delete`, `attach`, `detach` or `connect` in this family: registrations make
and attach dependency tools, agents reach them through their own tool list, and a tool's record goes with
its agent or with its dependency template's compartment (`postgres compartments delete`, in `diverge-storage`).

## What starts or holds a tool's container

| trigger | what it does to the tool's container |
|---|---|
| `create` | nothing |
| attach to an agent or template | `lazy`: nothing. `eager`: deployed, and held while attached |
| an agent's container starts and declares it (dependency) | deployed; `eager`: held while declared |
| an attached agent begins a loop | deployed; held while that agent is active |
| an MCP exchange into it | deployed; held while in flight |
| `standalone-tools connect` | deployed; held while the scope is open |
| `exec`, `terminal`, `frontend` | deployed; held while it runs |
| `filetree`, `download`, `upload`, `transfer` | deployed; held while in progress |
| a scope the tool itself has open on the daemon | held while open |
| nothing holding it for `idle_seconds` | undeployed |

Every row applies to standalone and dependency tools alike (a dependency tool has no `create`, attach or
`connect`). A dependency tool that mounts its agent's files brings its agent's container up with it and
holds it up. None of these makes a tool `active`: only an unanswered call or read does.

## Grants by leaf

| leaf | grant |
|---|---|
| `standalone-tools templates create` / `get` / `list` / `delete` / `tag` / `untag` | the same action over `standalone_tools_templates`; a `get` or `list` with `"arguments": true` also needs `arguments` |
| `standalone-tools create` | `create` over `standalone_tools`; plus `mount` over each volume named |
| `get`, `list`, `edit`, `delete`, `tag`, `untag`, `download`, `upload`, `transfer`, `filetree`, `frontend`, `connect` | the same action over `standalone_tools` |
| `exec`, `terminal` | `exec` |
| `attach-agent-instance`, `detach-agent-instance` | `attach` / `detach` over the tool and `edit` over the agent |
| `attach-agent-template`, `detach-agent-template` | `attach` / `detach` over the tool and `attach` over the agent template |
| reading a tool's log | `logs` over the tool |
| `dependency-tools get`, `list`, `tag`, `untag`, `download`, `upload`, `transfer`, `filetree`, `frontend` | the same action over `dependency_tools` (there is no making action: registrations make them); a `get` or `list` with `"arguments": true` also needs `arguments` |
| `dependency-tools exec`, `terminal` | `exec` over `dependency_tools` |
| a transfer's destination | `upload` over the agent, tool, dependency tool or volume it lands in |

Grant shapes, `within`, tags and diagnosing `"forbidden"` are in `diverge-access`.
