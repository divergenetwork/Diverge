# Every grant kind, every action

A grant is `{"<kind>": <permission>}`. The permission's shapes:

- **Make**: `["<make action>", …]`
- **Over**: `{"actions":["<over action>", …], "within": "any" | <filter>}`
- **Tagging**: `{"actions":["tag"|"untag", …], "within": "any" | <filter>, "tags": "any" | ["<tag>", …]}`
- **The one thing** (`postgres`, `events`, `images`): `["<action>", …]`

An administrator (an identity in the daemon's `admins` setting) is allowed everything and needs no grant.
`diverge run info` needs no grant. Print the authoritative schema with
`diverge run roles create request-schema --jq '."$defs"'`.

Command names below are `diverge run …` leaves.

## `within`: all, specific records, or tags

**One grant reaches all, exactly the records it names, or by tags. Never two of them.**

| Reach | `within` | Reaches |
|---|---|---|
| All | `"any"` | every record of the kind |
| Specific records | the kind's naming member, e.g. `{"ids":[…]}`, `{"volumes":[…]}` | exactly those records |
| Tags | `{"all_tags":[…]}`, `{"any_tags":[…]}` or both | records carrying every one of `all_tags` and any one of `any_tags` |

- **Naming records and tags in one grant is refused**, and so is naming records by two different members. The
  role create or edit, or the tool's registration, fails to decode.
- **"These records, or anything tagged so" is two grants.** Grants only add: a request is allowed when any one
  grant allows it, and no grant takes away what another gives.
- In a role's grant the kind's other filter members narrow whichever reach it is, and all must hold. A tool's
  grant carries the reach and nothing else.

What names one exact record, per kind. The same in a role's grant and in a tool's:

| Kind | Member | One item |
|---|---|---|
| `accounts` | `identities` | `{"issuer":…,"subject":…}` |
| `agents` | `agents` | `{"template":"<agent template id>","index":<n>}` |
| `agents_templates` | `ids` | the template's id |
| `standalone_tools` | `tools` | `{"template":"<tool template id>","index":<n>}` |
| `standalone_tools_templates` | `ids` | the template's id |
| `connected_standalone_tools` | `tools` | `{"daemon":"<daemon id>","tool":<the tool as that daemon names it>,"index":<n>}` |
| `dependency_tools` | `tools` | `{"template":"<dependency template id>","owner":<owner>}`; owner is `{"agent":{"template":…,"index":…}}` or `"template"` |
| `postgres_compartments` | `compartments` | a compartment as the daemon answers it: `{"instance":…}`, `{"template":{"kind":…,"template":…}}` or `{"image":{"digest":…}}`, an agent or a tool in it by template and index |
| `daemons` | `ids` | the remote daemon's id |
| `identities` | `identities` | `{"issuer":…,"subject":…}` |
| `roles` | `names` | the role's name |
| `volumes` | `volumes` | `{"provider":{"issuer":…,"subject":…},"name":"<name>"}` |
| `providers`, `providers_daemons`, `images` | `identities` | the provider, `{"issuer":…,"subject":…}` |

- `template`, `index`, a daemon's id and a tool's `tool` are what each record's list item shows.
- In a role's grant these also count as naming records, and so are never written with tags or with each other:
  `names` (agents, standalone tools, connected tools, volumes) and `subjects` (identities). A name can be given
  to another record later; the members in the table cannot.
- An empty list reaches nothing.

## agents

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `agents create` |
| Over | `get` | `agents get` |
| Over | `list` | `agents list` (sends what the grant reaches) |
| Over | `edit` | `agents edit`; also needed over the agent when a standalone or connected tool is attached to / detached from that one agent |
| Over | `delete` | `agents delete` |
| Over | `message` | `agents message` (a chat message and a structured decision alike) |
| Over | `logs` | `logs` with `of.type` = `agent` |
| Over | `download` | `agents download` |
| Over | `upload` | `agents upload`, and a transfer landing in the agent |
| Over | `transfer` | `agents transfer` (source side) |
| Over | `filetree` | `agents filetree` |
| Over | `exec` | `agents exec`, and `agents terminal` (a terminal is judged as `exec`) |
| Over | `frontend` | `agents frontend` |
| Over | `connect` | `agents connect …`: MCP exchanges run as the agent, with its tools, logged as its calls. Opening the scope deploys the agent and holds its container up while the scope is open. It does not make the agent active, writes nothing to its log, and does not block its edit or delete |
| Over | `request` | being asked by `requests create`: a request reaches the agent (named by itself, by a tag, or through `"all"`) only where this grant reaches it |
| Tagging | `tag`, `untag` | `agents tag`, `agents untag` |

`within` members: `agents` (exact), `names`, `templates`, `creators`, `active` (`true`/`false`, or `"ever"`/`"never"`),
`deployed`, `dependency_tool_templates`, `all_tags`, `any_tags`, `created_from`, `created_to`.

## agents_templates

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `agents templates create` |
| Over | `get`, `list`, `delete` | the `agents templates` request of the same name |
| Over | `arguments` | a `get` or `list` whose body says `"arguments": true` being answered with the template's arguments; held beside `get` / `list`, never instead |
| Over | `attach` | naming the template in `attach-agent-template` / `detach-agent-template` of a standalone or connected tool (beside the tool's own `attach`/`detach`) |
| Tagging | `tag`, `untag` | `agents templates tag` / `untag` |

`within`: `ids`, `creators`, `in_use`, `all_tags`, `any_tags`, `created_from`, `created_to`.

## standalone_tools

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `standalone-tools create` |
| Over | `get`, `list`, `edit`, `delete` | the request of the same name |
| Over | `attach` | `attach-agent-instance` (plus `edit` over the agent), `attach-agent-template` (plus `attach` over the template) |
| Over | `detach` | `detach-agent-instance`, `detach-agent-template` (same second grants) |
| Over | `download`, `upload`, `transfer`, `filetree` | the file requests; `upload` also covers a transfer landing in the tool |
| Over | `exec` | `standalone-tools exec` and `terminal` |
| Over | `frontend` | `standalone-tools frontend` |
| Over | `connect` | `standalone-tools connect …`: serving the tool's MCP exchanges to a client, **including another daemon that holds it as a connected tool** |
| Over | `logs` | `logs` with `of.type` = `standalone_tool` |
| Over | `request` | being asked by `requests create`: a request reaches the record (named by itself, by a tag, or through `"all"`) only where this grant reaches it |
| Tagging | `tag`, `untag` | `standalone-tools tag` / `untag` |

`within`: `tools` (exact), `names`, `templates`, `creators`, `modes`, `active`, `deployed`, `attached_agent_instances`,
`attached_agent_templates`, `all_tags`, `any_tags`, `created_from`, `created_to`.

## standalone_tools_templates

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `standalone-tools templates create` |
| Over | `get`, `list`, `delete` | the request of the same name |
| Over | `arguments` | a `get` or `list` whose body says `"arguments": true` being answered with the template's arguments; held beside `get` / `list`, never instead |
| Tagging | `tag`, `untag` | `standalone-tools templates tag` / `untag` |

`within`: `ids`, `creators`, `in_use`, `all_tags`, `any_tags`, `created_from`, `created_to`.

## connected_standalone_tools

Another daemon's standalone tool, held here under a name.

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `connected-standalone-tools create` (plus `relay` over the `daemons` record it names) |
| Over | `get`, `list`, `edit`, `delete` | the request of the same name |
| Over | `attach` | `attach-agent-instance` (plus `edit` over the agent), `attach-agent-template` (plus `attach` over the template) |
| Over | `detach` | `detach-agent-instance`, `detach-agent-template` |
| Over | `logs` | `logs` with `of.type` = `connected_standalone_tool` |
| Tagging | `tag`, `untag` | `connected-standalone-tools tag` / `untag` |

`within`: `tools` (exact), `names`, `daemons` (record IDs), `creators`, `active`, `deployed`, `attached_agent_instances`,
`attached_agent_templates`, `all_tags`, `any_tags`, `created_from`, `created_to`.

## dependency_tools

No making shape: a dependency tool is made by an agent's registration, never by a request.

| Shape | Action | Allows |
|---|---|---|
| Over | `get`, `list` | `dependency-tools get` / `list` |
| Over | `arguments` | a `get` or `list` whose body says `"arguments": true` being answered with the declared template's arguments; held beside `get` / `list`, never instead |
| Over | `download`, `upload`, `transfer`, `filetree` | the file requests on a running tool |
| Over | `exec` | `dependency-tools exec` and `terminal` |
| Over | `frontend` | `dependency-tools frontend` |
| Over | `logs` | `logs` with `of.type` = `dependency_tool` |
| Over | `request` | being asked by `requests create`: a request reaches the record (named by itself, by a tag, or through `"all"`) only where this grant reaches it |
| Tagging | `tag`, `untag` | `dependency-tools tag` / `untag` |

`within`: `tools` (exact), `ids` (of these templates), `owners`, `creators`, `modes`, `scopes`, `depended`, `agents`, `in_use`, `all_tags`,
`any_tags`, `created_from`, `created_to`.

## providers

`within` here is `"any"` or a **provider reach**: `{"identities":[{"issuer":…,"subject":…}],"all_tags":[…],"any_tags":[…]}`,
each member optional. A grant names providers by `identities`, or by tags (every one of `all_tags` and any one
of `any_tags`), and never both in one grant: that is refused. Write a grant for each. No other member decodes. The same reach is the `within` of
`providers_daemons` and of `images`.

| Shape | Action | Allows |
|---|---|---|
| Make | `add` | `providers add` |
| Over | `get`, `list` | `providers get` / `list` |
| Over | `edit` | `providers edit` |
| Over | `enable`, `disable` | `providers enable` / `disable` |
| Over | `delete` | `providers delete` |
| Over | `read` | `images read` (whether an image can be supplied, and its manifests, blobs, proof and DIVERGE.md): the daemon asks only the providers the grant reaches |
| Tagging | `tag`, `untag` | `providers tag` / `untag` |

## providers_daemons

What is done *through* providers to the daemons connected to them. Over shape only; `within` is `"any"` or
a provider reach (identities, tags, or both: see `providers` above).

| Action | Allows |
|---|---|
| `list` | `providers daemons list`: only routes through reached providers are listed |
| `listen` | `providers daemons listen`: only messages that came through reached providers |
| `message` | `providers daemons message`: sent through no provider the grant does not reach |

Examples: `{"providers_daemons":{"actions":["list","listen","message"],"within":"any"}}`,
`{"providers_daemons":{"actions":["list"],"within":{"identities":[{"issuer":"…","subject":"ronald.provider"}]}}}`.

## images

The daemon's Stores: the images it owns on each provider. Over shape only; `within` is `"any"` or a provider
reach (identities, tags, or both: see `providers` above). **A bare array, `{"images":["build"]}`, no longer
decodes.**

| Action | Allows |
|---|---|
| `build` | `images build` on a provider the grant reaches (plus `mount` over the context volume, where one is named) |
| `push` | `images push` to a provider the grant reaches. Its `source` needs no grant: `daemon` has the daemon find the image as itself, as it does for a run |
| `get` | `images get` of an image in the Store on a provider the grant reaches |
| `delete` | `images delete` of an image, or some of its platforms, in the Store on a provider the grant reaches |
| `merge` | `images merge` done on a provider the grant reaches. Where the images are brought from needs no grant |
| `publish` | `images publish` done on a provider the grant reaches: the one named, or one the daemon picks from those reached. Where the parts are brought from needs no grant |
| `list` | `images list`: only the Stores on reached providers are listed |

Every action is judged by the provider alone, never by the image's digest. The Store on a provider is the
daemon's: every account whose grant reaches the provider sees and adds to the same Store. `images read`, an
image's DIVERGE.md included, is not held here: it is judged by `read` over providers and `relay` over daemons.

Examples: `{"images":{"actions":["build","push","list"],"within":"any"}}`,
`{"images":{"actions":["build"],"within":{"all_tags":["build"]}}}`.

## daemons

Remote daemons' records.

| Shape | Action | Allows |
|---|---|---|
| Make | `add` | `daemons add` |
| Over | `get`, `list`, `edit`, `delete` | the request of the same name |
| Over | `relay` | **using** the remote daemon, whatever the use: any request whose `relay` starts with the record's ID; naming the record as `daemon` in `connected-standalone-tools create`; naming it as `daemon` in a container's FUSE mount; `images read`, which asks only the remote daemons the grant reaches |
| Tagging | `tag`, `untag` | `daemons tag` / `untag` |

`relay` is the one permission to use a remote daemon. There is no `connect` or `read` over daemons: a role
that still lists either does not decode and must be edited to `relay`. What the remote daemon then allows
the account the record names is the remote's own to judge.

`within`: `ids`, `issuers`, `creators`, `connected`, `all_tags`, `any_tags`, `created_from`, `created_to`.

## accounts

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `accounts create` (plus `grant` over each role named) |
| Over | `get`, `list`, `delete` | the request of the same name |
| Over | `edit` | `accounts edit` (plus, when the roles change, `grant` over every role in the resulting list) |
| Over | `logs` | `logs` with `of.type` = `account`: the account's command log |
| Over | `disconnect` | `accounts disconnect` |
| Over | `request` | being asked by `requests create`: a request reaches the record (named by itself, by a tag, or through `"all"`) only where this grant reaches it |
| Tagging | `tag`, `untag` | `accounts tag` / `untag` |

`within`: `identities`, `roles`, `connected`, `creators`, `all_tags`, `any_tags`, `created_from`,
`created_to`. An administrator is reached through `identities`, `connected` and its tags.
An administrator's command log is read like any account's (`of.type` = `account`, with `logs` over
accounts reaching it); the older `of.type` = `admin` form is read by administrators alone.

## roles

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `roles create` |
| Over | `get`, `list`, `edit`, `delete` | the request of the same name |
| Over | `grant` | naming the role in an account's `roles` at `accounts create` / `accounts edit` |
| Tagging | `tag`, `untag` | `roles tag` / `untag` |

`within`: `names`, `accounts`, `creators`, `all_tags`, `any_tags`, `created_from`, `created_to`.

Note the escalation paths: `edit` over a role lets the holder rewrite the grants of a role it may itself
hold; `create` over roles plus `grant` within `"any"` plus `edit` over accounts is full control. Scope
`grant` and `edit` with `within`.

## volumes

| Shape | Action | Allows |
|---|---|---|
| Make | `create` | `volumes create` |
| Over | `get`, `list`, `edit`, `delete`, `stat` | the request of the same name |
| Over | `download`, `upload`, `transfer`, `filetree` | the file requests; `upload` also covers a transfer landing in the volume |
| Over | `mount` | naming the volume in a container's mounts at its create or edit (beside the container's grant) |
| Over | `fuse_serve` | `volumes fuse_serve`: having the volume served over FUSE to the caller. What **another daemon's account** needs here so that its containers can FUSE-mount the volume. `mount` does not imply it |
| Tagging | `tag`, `untag` | `volumes tag` / `untag` |

`within`: `volumes` (exact), `providers`, `names`, `modes`, `mounted`, `all_tags`, `any_tags`, `created_from`, `created_to`.
A grant reaches a volume or a container whole; it cannot narrow by path.

## postgres, events (bare arrays)

| Kind | Actions | Allows |
|---|---|---|
| `postgres` | `get`, `list` | `postgres get` (which database, no password), `postgres list` (container connections) |
| `events` | `list` | `events list` |

Examples: `{"postgres":["get","list"]}`, `{"events":["list"]}`.

## postgres_compartments

No making shape: a compartment is on record when its agent, tool or dependency tool is made.

| Shape | Action | Allows |
|---|---|---|
| Over | `get`, `list` | `postgres compartments get` / `list` |
| Over | `delete` | `postgres compartments delete`. Deleting a dependency template's compartment also removes that template's tools; this one grant covers it |
| Tagging | `tag`, `untag` | `postgres compartments tag` / `untag` |

`within`: `compartments` (exact), `databases`, `kinds`, `templates`, `images`, `deletable`, `all_tags`,
`any_tags`, `created_from`, `created_to`.

Example: `{"postgres_compartments":{"actions":["get","list"],"within":"any"}}`.

## identities

The identities signed in on the daemon's host.

| Shape | Action | Allows |
|---|---|---|
| Make | `login` | `identities login` of an identity not on the host yet |
| Over | `get` | `identities get` |
| Over | `list` | `identities list` |
| Over | `login` | `identities login` replacing an identity already on the host |
| Tagging | `tag`, `untag` | `identities tag` / `untag` |

`within`: `identities` (exact), `issuers`, `subjects`, `all_tags`, `any_tags`.

## A read-only "observer" role

```json
{"name":"observer","grants":[
 {"agents":{"actions":["get","list","logs"],"within":"any"}},
 {"agents_templates":{"actions":["get","list"],"within":"any"}},
 {"standalone_tools":{"actions":["get","list","logs"],"within":"any"}},
 {"standalone_tools_templates":{"actions":["get","list"],"within":"any"}},
 {"connected_standalone_tools":{"actions":["get","list","logs"],"within":"any"}},
 {"dependency_tools":{"actions":["get","list","logs"],"within":"any"}},
 {"providers":{"actions":["get","list","read"],"within":"any"}},
 {"providers_daemons":{"actions":["list"],"within":"any"}},
 {"daemons":{"actions":["get","list"],"within":"any"}},
 {"accounts":{"actions":["get","list"],"within":"any"}},
 {"roles":{"actions":["get","list"],"within":"any"}},
 {"volumes":{"actions":["get","list","stat"],"within":"any"}},
 {"postgres":["get","list"]},
 {"postgres_compartments":{"actions":["get","list"],"within":"any"}},
 {"events":["list"]},
 {"identities":{"actions":["list"],"within":"any"}}]}
```

## Grants a container's image declares

A container is not an account. A tool's container and an agent's container each hold the grants their
program declared at registration, declared again at every start, written in the same vocabulary; an agent
may declare any grant a tool may. For every kind, `within` is one of the three reaches above and nothing
else: `"any"`, the kind's naming member from the table (`{"ids":[…]}`, `{"volumes":[…]}`,
`{"identities":[…]}`, …), or tags. A declared grant has no narrowing members.

- An image never fixes a list of records: a record is one daemon's. A program whose **arguments** name
  records (the agents to message, the daemon to relay to, the volume to read) declares those, so the same
  image declares one list here and another there.

Four kinds have one word more, `"attached"`, in the `within` of an Over grant. It reaches the records on
the other side of an attachment from the container that holds the grant, and nothing on its own side:

| the requester | `"attached"` over `agents` | `"attached"` over `standalone_tools`, `connected_standalone_tools`, `dependency_tools` |
|---|---|---|
| a tool | the agents attached to it (for a dependency tool, the agents whose latest registration declared it) | nothing |
| an agent | nothing | the tools served to it: its dependency tools, and the standalone and connected tools attached to it or to its template |

- It is judged at every request, whether or not what it reaches is running.
- An action held only within `"attached"` is held at all only while something of that kind is attached: a
  tool with no attached agent that lists agents on such a grant alone is answered `"forbidden"`.
- Over agents alone there is one word besides, `"never_active"`: agents that have never run.

Neither word is valid in a role's grant. A container, having no account, cannot use `frontend` or
`standalone-tools connect` whatever it declares. See `diverge-tools`, `diverge-agents` and `diverge-images`.
