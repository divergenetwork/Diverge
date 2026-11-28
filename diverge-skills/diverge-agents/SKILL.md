---
name: diverge-agents
description: Diverge agents through the `diverge run agents` commands — agent templates (`agents templates create|get|list|delete|tag|untag`; the template body with image, extra_memory, extra_disk, arguments; the required `arguments` choice on `get` and `list`; the template id as a hash), agents (`agents create|get|list|edit|delete|tag|untag`, provider pin, volume mounts, FUSE mounts), asking an agent something with `agents message` (`type` `chat`: content blocks, delivery order, idle stop and restart; `type` `structured_decisions`: a prompt and typed questions — noul, choice, score — answered with numbers in the log; `watch` to get back the agent's log for that request), MCP as an agent (`agents connect list-tools|call-tool|list-resources|read-resource`), files in an agent's container (`agents filetree|download|upload|transfer`), `agents exec`, `agents terminal`, `agents frontend`, agent references (`{"name":…}` vs `{"template":…,"index":…}`), and the agent lifecycle its log shows (deployed, active, inactive, undeployed). Load it when asked to make, run, message, inspect, change, debug or delete a Diverge agent or agent template.
---

# Diverge agents

An **agent** is one container the daemon runs on a provider: an agent-loop program (a model harness, a
script, or a person at a frontend) that takes messages and answers them, calling the MCP tools the daemon
serves it. An agent is made from an **agent template** (image, limits, arguments), which is
held by its hash, plus what is the agent's own: a name, a provider and mounts. The daemon keeps the agent's record and its whole conversation (its log); the container itself only runs
while there is work for it.

Everything here is one request per command: `diverge run agents <leaf> '<JSON body>'`. Every answer is one
compact JSON line. Run `<leaf> request-schema` or `<leaf> response-schema` for exact shapes; neither dials a
daemon. Every body may also carry `relay` (see `diverge-daemons`).

```bash
diverge run agents --help
diverge run agents create request-schema
```

## Order of operations

1. An image is in the provider's store and you know its digest (`diverge-images`).
2. `agents templates create` → the template id. Any credential the image needs goes in its `arguments`.
3. `agents create` → the agent (nothing runs yet).
4. Attach tools if wanted (`diverge-tools`).
5. `agents message` → the container starts, the agent works, the container is stopped when idle.
6. Read what it said with `diverge run logs` (`diverge-logs`).

## Naming an agent: two references

Every request that acts on an agent takes `"agent": <reference>`, and you choose the form each time:

- `{"name":"echo"}` — the name its create gave it. Names are optional, unique among the caller's agents, and
  free again after a delete. An agent made without a name cannot be reached this way.
- `{"template":"<template id>","index":3}` — the template it was made from and its number among all agents
  ever made from that template (first is `1`, never reused, deleted ones included). This names the agent once
  and for all and always works. `agents list` reports both members on every item.

An object with members of both forms is malformed. A template is always named by its `id` alone.

## Agent templates

A template is everything every agent made from it shares. It names no provider, no mounts and no name, so it
can be shared between daemons and hashes the same anywhere.

```bash
diverge run agents templates create '{
  "type": "agent",
  "image": {"digest": "sha256:b031dd…", "references": [{"registry": "docker.io", "name": "diverge-local/python"}]},
  "extra_memory": 0,
  "extra_disk": 0,
  "arguments": {"python": "…script source…"}
}'
```

| member | meaning |
|---|---|
| `type` | `"agent"`. Required; a tool template (`"type":"tool"`) is a different namespace and hashes apart. |
| `image` | `{"digest":"<algorithm>:<hex>","references":[{"registry":…,"name":…}]}`. The digest is the image. References only say where the bytes may be fetched and are not in the hash. The daemon refuses to start a run unless a reference names a registry the provider lists, so include one even for a locally built image. The image must be an agent image: its `DIVERGE.md` says `"type":"agent"`. |
| `extra_memory` | Bytes, required: the memory the container gets beyond what its image's `DIVERGE.md` says it needs. The container's limit is the sum of the image's `memory` and this. `0` is what the image needs and no more. The limit is a hard ceiling: a process over it is killed. |
| `extra_disk` | Bytes, required: what the container may write over its image beyond what the image's `DIVERGE.md` says it needs. The limit is the sum of the image's `disk` and this. `0` is what the image needs and no more. Mounted volumes are not counted. |
| `arguments` | Any JSON value, handed to the image's program verbatim. Its shape is the image's own: read the image's `DIVERGE.md` (`diverge run images read --info`, see `diverge-images`). A credential the image needs is a member of them (the `openrouter` image's `api_key`), so anyone who may see the template's arguments sees it. |

A template has exactly these five members. It has no `description`, no `memory` and no `disk`: the image's
own `DIVERGE.md` states its description and the memory and disk it needs.

Answers: `{"created":"<id>"}`, `{"exists":"<id>"}` (the same template was made before; not a failure),
`"forbidden"`, `{"error":…}`. The image is not checked here; `agents create` checks it, and refuses a template whose image is not an agent image. The **id** is the lowercase hex SHA-256 of the template's canonical JSON
(64 characters). Capture it:

```bash
TPL="$(diverge run agents templates create --jq '.created // .exists' "$(cat template.json)" | tr -d '"')"
```

A template cannot be edited: a changed body is a new template with a new id.

**Dependencies are not a template member.** The tools an agent depends on are declared by the agent's
*program* when its container registers. For the agent-loop images in this repository that declaration is
copied from `arguments.dependencies`, a list of dependency tool templates (image, extra_memory, extra_disk,
arguments; the tool image's own `DIVERGE.md` states its `mode` of `lazy` or `eager` and its `scope` of `instance` or `template`; a tool
whose image says lazy and `instance` may also mount the agent's own paths). Each image must be a `dependency_tool` image. The daemon deploys them when the agent's container starts; each is then held and undeployed by its own idle time, and the agent's next loop deploys again any that went down.
See `diverge-tools` for the full shape and what each mode and scope means.

Other template leaves:

| command | body | answers |
|---|---|---|
| `agents templates get` | `{"id":"<id>","arguments":false}` | `{"found":{id, created, creator, tags, tools, template}}`, `"not_found"`, `"forbidden"` |
| `agents templates list` | `{"arguments":false}`, plus any of the filter: `ids`, `creators`, `in_use`, `all_tags`, `any_tags`, `created_from`, `created_to`, `count` | one `{"added":…}` per template, then `"listed"`; `"forbidden"` |
| `agents templates delete` | `{"id":"<id>"}` | `"deleted"`, `"not_found"`, `"in_use"` (an agent was made from it: delete those agents first) |
| `agents templates tag` | `{"id":"<id>","tags":["crew"]}` | `"tagged"`, `"not_found"` |
| `agents templates untag` | `{"id":"<id>","tags":["crew"]}` | `"untagged"`, `"not_found"` |

**`arguments` is required on `get` and `list`**, `true` or `false`: a body without it, a bare `list`
included, is refused. Arguments carry credentials, so say `false` unless you need them.

```bash
diverge run agents templates list '{"arguments":false}'
diverge run agents templates get '{"id":"<id>","arguments":true}'
```

- `false`: the answer's `template` has no `arguments` member. Nothing more than `get` / `list` is needed.
- `true`: the `template` carries its `arguments`. This needs the `arguments` grant over the template, beside
  `get` / `list`. A `get` without it answers `"forbidden"`. A `list` answers `"forbidden"` unless `arguments`
  grants reach **every** template it would show; kept open with `--watch`, it does not show a later template
  outside that reach.

A listed template's `tools` are the tools attached to the template (every agent made from it has them).
Tags are the caller's, not in the hash. Deleting a template removes its tool attachments with it.

## Creating an agent

```bash
# minimal: runs on whichever provider the daemon chooses, mounts nothing
diverge run agents create "{\"template\":\"$TPL\",\"name\":\"echo\"}"

# fuller: pinned to a provider, one volume mounted
diverge run agents create '{
  "template": "<id>",
  "name": "researcher",
  "provider": {
    "identity": {"issuer": "https://identity.diverge.network", "subject": "ronald.provider"},
    "volume_mounts": [
      {"volume_name": "notes", "volume_relative_path": [], "volume_mode": "persistent", "container_path": ["data"]}
    ]
  }
}'
```

- `template` (required): an agent template id of the caller's. A tool template's id is an error.
- `name` (optional): unique among the caller's agents; taken → `"in_use"`.
- `provider`: pins the agent to one provider (by `{"issuer","subject"}`) and mounts that provider's volumes.
  Absent, the daemon picks a provider at each start and the agent mounts no volume.
- `fuse_file_mounts`, `fuse_directory_mounts`: files or directories of any provider's volume, served live
  through the daemon. A mount that names a `daemon` reaches a volume on another daemon.

Mount shapes, rules and edit patches are in [references/mounts.md](references/mounts.md). There is no
environment, entrypoint or port to set: the mounts and the template's arguments are the only provisioning.

Answers: `"created"`, `"in_use"`, `"forbidden"`, `{"error":…}`. **A create starts nothing.** The agent has no
account. Its program may still put requests to the daemon over its own daemon connection, as a tool's may,
within the permissions the program declares each time its container starts: see "An agent's permissions".

## Getting and listing

```bash
diverge run agents get '{"agent":{"name":"echo"}}'            # {"found":{…}} or "not_found"
diverge run agents list                                        # every agent, then "listed"
diverge run agents list '{"active":true,"all_tags":["crew"]}'
diverge run agents list --watch '{"templates":["<id>"]}'       # stays open: added / changed / removed
```

A list prints `{"added":{…}}` per agent, oldest first, then `"listed"`, and ends there unless `--watch`.
An item carries `name`, `template`, `index`, `creator`, `created`, `active`, `last_active`, `deployed`,
`last_deployed`, `provider`, `logs_index` (length of its log), `tools` (attached standalone and connected
tools), `dependencies` (dependency template ids its last registration declared), `permissions` (the grants
its program declared at its last start; absent when none or never started) and `tags`.

Filter members (all optional; lists match any one of, except `all_tags`): `names`, `templates`, `creators`,
`active` (`true`, `false`, `"ever"`, `"never"`), `deployed`, `dependency_tool_templates`, `all_tags`,
`any_tags`, `created_from`, `created_to`, and `count` (keep to the first N that match).

`active` means a loop is running, or an MCP exchange the agent's own container asked for is in flight.
Nothing else makes an agent active, and from outside only a message does: not a file operation, not a
`frontend` request, not `agents connect`. `deployed` means the container is up. An agent can be deployed
and inactive (held by a use, or idling before it is stopped).

## Editing

An edit is a JSON Patch (RFC 6902) applied to the agent's editable document: `name`, `volume_mounts`,
`fuse_file_mounts`, `fuse_directory_mounts`. Applied whole or not at all.

```bash
diverge run agents edit '{"agent":{"name":"echo"},"patch":[{"op":"replace","path":"/name","value":"echo-2"}]}'
```

Answers: `"edited"`, `"not_found"`, `"in_use"` (the new name is another agent's), `"active"`, `"forbidden"`,
`{"error":…}`.

- While the container is **running**, the only change accepted is FUSE mounts appended to either list
  (`"path":"/fuse_file_mounts/-"`); they are mounted into the running container. Anything else answers
  `"active"`: wait until the agent is undeployed (about `idle_seconds` after its last use) and ask again.
- The template, image, limits, arguments and pinned provider never change. For different arguments, make a
  new template and a new agent.
- An agent pinned to no provider cannot be given `volume_mounts`.

## Deleting, tagging

```bash
diverge run agents delete '{"agent":{"name":"echo"}}'   # "deleted" | "not_found" | "active"
diverge run agents tag    '{"agent":{"name":"echo"},"tags":["crew","blue"]}'   # "tagged"
diverge run agents untag  '{"agent":{"name":"echo"},"tags":["blue"]}'          # "untagged"
```

- Delete is irreversible: the container is stopped, the record goes, the name is free. Its
  `{"template","index"}` is never given to another agent. An agent that is active answers `"active"`
  and is left alone; so does one whose dependency tool has a call or a read unanswered. Wait and retry.
  Its instance-owned dependency tools go with it.
- Tags are plain strings. Tagging an existing tag, or untagging an absent one, is not a failure. Tags are
  what grants and filters select by (`diverge-access`).

## Messaging

One command, `agents message`, asks an agent one of two things, by the body's `type`, both under the one
`message` grant:

- `"type":"chat"`: a message that joins the agent's conversation. Everything below, up to
  [Structured decisions](#structured-decisions), is about chat.
- `"type":"structured_decisions"`: a prompt and typed questions, answered with numbers.

Add `"watch":true` to either to get the agent's log for that request back in place of the one-line fate:
see [Watching](#watching).

```bash
diverge run agents message '{"agent":{"name":"echo"},"type":"chat","content":[{"type":"text","text":"hello"}]}'
# "delivered"
```

`content` is an ordered list of MCP content blocks, relayed to the agent's program verbatim:

| block | shape |
|---|---|
| text | `{"type":"text","text":"…"}` |
| image | `{"type":"image","data":"<base64>","mimeType":"image/png"}` |
| audio | `{"type":"audio","data":"<base64>","mimeType":"audio/wav"}` |
| resource | `{"type":"resource","resource":{"uri":"file:///a.txt","mimeType":"text/plain","text":"…"}}` (or `"blob":"<base64>"` in place of `text`) |
| resource link | `{"type":"resource_link","uri":"https://…","name":"…"}` |

Rules:

- **One request, one answer** (unwatched): `"delivered"`, `"forbidden"`, `"no_chat"`,
  `{"unsupported_modalities":{"prompt":[…],"instruction":[],"choice":[]}}` or `{"error":…}`. `delivered` means the agent's
  program accepted the message (the loop it started took it, or it went onto the program's own queue). It
  does not mean the agent has answered. The answer is in the log.
- **Nothing takes a message back.** There is no cancel and no dequeue.
- **The first message starts the container, and every tool the agent is served**: the dependency tools its
  program declares (either scope, lazy or eager), the standalone tools attached to it or to its template,
  and the connected tools. They are started together once the program has registered, and all are waited
  for. **If any one cannot be deployed, the agent's start fails**, naming that tool, and the message answers
  that `{"error":…}`. The same start, with the same rule, is what a file operation, `exec`, `terminal`,
  `frontend` or `agents connect` runs on an agent that is down.
- **A message for an agent that is already up** first brings back any tool it is served that has gone down
  since. One that cannot be brought back fails the message, and nothing is delivered.
- **The agent holds its tools** while it is starting (from its registration), while a message is on its way
  into it, and while it is active. A start for anything but a message lets them go when it finishes, and
  each then idles out by itself.
- The request waits for all of that, however long it takes; use `--timeout` if you need a bound. An
  abandoned request does not recall the message.
- **Order**: messages to one agent are delivered in the order one connection sent them. Each CLI command is
  its own connection, so run commands one after another when order matters.
- **Joining**: messages that are waiting while the agent starts are joined, in order, into its first turn as
  one user message. Text parts stay separate blocks.
- **Idle stop**: once the agent is inactive and unused for the daemon's `idle_seconds` (default 10), its
  container is stopped. A scope the agent's own program has open on its daemon connection counts as use:
  it holds the container up without making the agent active, an endless stream included. The record, the conversation and anything on mounted volumes persist; the next
  message starts it again. Files written only to the container's own disk are gone with the container.
- **What the agent takes**: the daemon holds the message to the `chat` of the agent's image's `DIVERGE.md`
  before sending anything. `"no_chat"`: the image does not chat, and no message is taken.
  `{"unsupported_modalities":{"prompt":["audio"],"instruction":[],"choice":[]}}`: the message carries an image, audio or a file (a resource with
  `blob` that is neither `image/*` nor `audio/*`) that the image's `prompt_modalities` says `false` to; the
  whole message is refused. Either way nothing was started and the agent's log has no line of it. Text, a
  text resource and a resource link are always taken.
- **Errors**: no such agent is an `{"error":…}` here (not `"not_found"`); so is an image that refuses a block
  kind it cannot take (the whole message is refused), and a run that could not start (arguments the image refuses,
  a dependency that did not deploy, no capable provider). The same failure is kept in the agent's log.
- A first message that fails with "connection ended before the channel answered" usually means dependency
  tools were still deploying. Send it again.

### Structured decisions

```bash
diverge run agents message '{
  "agent": {"name": "judge"},
  "type": "structured_decisions",
  "content": [{"type":"text","text":"Review this pull request."}],
  "questions": [
    {"id":"risk","type":"noul","instructions":[{"type":"text","text":"How risky is merging it, from 0 to 1?"}]},
    {"id":"verdict","type":"choice","instructions":[{"type":"text","text":"What should happen?"}],
     "options":[{"value":"merge"},{"value":"revise","description":[{"type":"text","text":"Send it back with comments."}]}]},
    {"id":"quality","type":"score","instructions":[{"type":"text","text":"Rate the code."}],
     "levels":[[{"type":"text","text":"Poor"}],[{"type":"text","text":"Fine"}],[{"type":"text","text":"Excellent"}]]}
  ]
}'
# "delivered"
```

`content` is the prompt, the same blocks as chat. Each question has an `id` (the key its answer comes back
under), a `type`, and `instructions` (content blocks):

| `type` | adds | answered with |
|---|---|---|
| `noul` | nothing | `{"noul": 0.25}` |
| `choice` | `options`: each `{"value": "<text>", "description"?: [blocks]}` | `{"choice": "<value>", "probabilities": {"<value>": 0.7, …}}` |
| `score` | `levels`: each a list of blocks | `{"score": 1.7, "probabilities": [0.1, 0.2, …]}` |

`choice` and `score` are **computed by the daemon** from the probabilities when it writes the answer to the
agent's log; the agent's program states only probabilities:

- `choice` is the option with the highest probability. On a tie, the one that sorts first alphabetically
  (byte order). With no probabilities, `""`.
- `score` is the sum of each level's probability times its position, **the first level counting 0**. Three
  levels with probabilities `[0.1, 0.2, 0.7]` score `0.2×1 + 0.7×2 = 1.6`.
- Nothing is checked against the question: probabilities that do not sum to 1, a key that is no option, or
  a wrong number of levels are computed as given.

Rules:

- **Unwatched, the answer is the decision's fate, not the agent's answers**: `"delivered"`, `"forbidden"`,
  `"no_structured_decisions"`, `{"unsupported_modalities":{"prompt":[…],"instruction":[…],"choice":[…]}}` or `{"error":…}`.
  The agent's answers are in its log (see `diverge-logs`); `"watch":true` streams them to you.
- **The image must declare them.** `"no_structured_decisions"` when the image's `DIVERGE.md` says
  `structured_decisions: false` (every image in this repository does, so nothing here answers one yet).
- **The questions are validated before anything is sent**; a fault is
  `{"error":{"kind":"questions","question":"<id>","error":"…"}}`:
  - at least one question; ids non-empty and distinct;
  - `instructions` non-empty;
  - a choice has at least two options, each `value` non-empty and distinct within the question, each
    `description` absent or non-empty;
  - a score has at least two levels, each non-empty.
  - "Non-empty" means at least one block that is not an empty text block. Whitespace counts and nothing
    is trimmed; any image, audio, resource or link counts.
- **Modalities**: `content` is held to the image's `structured_decisions.prompt_modalities`; every
  question's `instructions` to its `instruction_modalities`; every option `description` and every score
  level to its `choice_modalities`. The answer lists what was carried and not taken, by where.
- **Order**: chat messages and decisions to one agent are taken in the order they were sent, whichever
  each is. A decision is not queued onto a running turn: it waits until the agent has finished everything
  before it and is idle, runs alone, and finishes before anything after it starts. So `"delivered"` for a
  decision can come long after it was sent, and a chat sent after a decision waits for the decision to
  finish.
- Nothing takes a decision back.

### Watching

`"watch":true` ties what you sent to what the agent did with it. The command then prints the part of the
agent's log that belongs to your request, one line per entry, in the same `{"item":{"agent":{…}}}` shape
`diverge run logs` prints, and exits by itself:

```bash
diverge run agents message '{"agent":{"name":"echo"},"type":"chat","watch":true,
  "content":[{"type":"text","text":"hello"}]}' \
  --jq '.item.agent | select(.type=="assistant_text_content") | .text'
```

- **The first line is your request, echoed**: for chat, its first `user_*` entry; for a decision, its
  `user_structured_decisions` entry (its `id` is the one the answer carries). No `"delivered"` is printed.
- **Then every entry the log takes after it**, yours or not (the agent's text, tool calls, errors, another
  sender's message that joined the same run), through the last:
  - chat: the first `inactive` after the echo, which is the end of the run your message landed in;
  - decision: the `structured_decisions_response` with the echoed decision's `id`. If the agent never
    answers, the first `inactive`. The last line's `type` tells you which.
- **Nothing is printed before the echo.** A message waiting for the container to start, or behind a
  decision, prints nothing until it is taken up; entries from runs before its own are not yours and are
  not printed. A chat that joins a run already going gets that run from its echo on, not from its start.
- **A request that never reaches an echo prints one line**: the refusal or `{"error":…}` it would have had
  unwatched (a run that could not start is an `{"error":…}` with the start's own reason), or `"dropped"`.
- **`"dropped"`**: the agent's program accepted the request and its run ended (or went idle twice) without
  ever echoing it. It will not show up later: a request belongs to the one run it was delivered into.
  Treat it as not handled and send it again.
- Stopping the command (or `--timeout`) ends the watch only. The request still goes to the agent.

Read the reply:

```bash
diverge run logs '{"of":{"type":"agent","name":"echo"}}'
diverge run logs '{"of":{"type":"agent","name":"echo"},"watch":true,"type":"assistant_text_content"}'
```

## The lifecycle, as the log shows it

Only the daemon writes these four items, in this order, per run of the container:

```
deployed            container is up on a provider (names the provider)
  active            a loop began, or an exchange the agent's own container asked for is in flight
    user_text_content / assistant_* / tool_response / usage …
  inactive          the work ceased; the idle clock runs from here
  active … inactive (again, for each further stretch of work)
undeployed          container is down: {"interrupted":false} and no "reason" when it was asked to stop
```

- `undeployed` with a `reason` is an unexpected end: `provider_lost`, `provider_disabled`,
  `provider_edited`, `daemon_stopped`, `daemon_died` (the daemon was killed; the entry was written at its next start, so its time is the restart's), `mount_lost` (a FUSE mount lost its serve), `exited` (the container exited by itself) or `failed` (an `error`
  item just before says how). `interrupted: true` means it was working when it went down.
- `deployed` carries `cause`, what found the container down and started it: `message`, `connect` (an `agents connect` scope opening), `dependency_tool`, or one of `download`, `upload`, `transfer`, `filetree`, `exec`, `terminal`, `frontend`.
  `dependency_tool` means a dependency tool that mounts this agent's files was deployed while the agent was
  down; that tool holds the agent's container up for as long as the tool is up.
- A run that never came up keeps an `error` item and no `deployed`.
- A `command` item (`path`, `request`, `allowed`) is one request the agent's own container put to the
  daemon. It makes the agent neither active nor inactive.
- Each user part carries `sender` (who sent the message) and `key` (which message it belongs to).

`diverge-logs` covers the log read in full (spans, `type`, `count`, `watch`).

## MCP as an agent: `agents connect`

Runs one MCP exchange exactly as the agent's own container would: the same merged tool list, the same names,
logged in each tool's log as the agent's call. Use it to see what an agent can call, or to drive a tool by
hand as that agent. Body is `{"agent":<reference>}`; the exchange is the subcommand. Each CLI command opens
one connect scope, runs its one exchange and closes it.

```bash
diverge run agents connect list-tools '{"agent":{"name":"echo"}}'
diverge run agents connect call-tool --name b6dd4f61_remember --arguments '{"text":"hi"}' '{"agent":{"name":"echo"}}'
diverge run agents connect list-resources '{"agent":{"name":"echo"}}'
diverge run agents connect read-resource --uri 'memo://1' '{"agent":{"name":"echo"}}'
```

- Output is the MCP result's own JSON. `list-tools` and `list-resources` print one page; pass the result's
  `nextCursor` as `--cursor` for the next.
- Tool names are `<prefix>_<tool name>`, the prefix naming which attached tool serves it (see
  `diverge-tools`). Each listed tool also says who serves it under `_meta` → `diverge.network`.
- **Opening the scope deploys the agent** if its container is down (the ordinary start, with every tool it
  is served) and **holds the container up for as long as the scope is open**, with or without an exchange in
  flight. The idle countdown starts when the scope closes.
- **Nothing on it makes the agent active** or keeps it active, and nothing of it is written to the agent's
  log. An agent held only by a connect can be edited or deleted; the scope then ends.
- An exchange holds the tool it reaches while it is in flight, and deploys that tool if it is down.
- The scope ends when the client disconnects, when its connection ends, or when the agent's container ends.
  A container that ended is not started again under an open scope.
- The connect itself answers `"not_found"`, `"forbidden"` or `{"error":…}` when it cannot be opened; a
  container that will not start, a tool it is served that cannot be deployed among the reasons, is that
  `{"error":…}`.
- The wire has a fifth exchange, the notifications stream. The CLI does not carry it.

## Files in the container

All paths in bodies are arrays of components from the container's root (`["workspace","out.txt"]`), never
strings with `/`. No component may be empty, `.` or `..`.

```bash
# the whole tree, once (add --watch to keep getting changes)
diverge run agents filetree '{"agent":{"name":"echo"}}'

# download a file, or a directory (path [] is the whole root)
diverge run agents download --path ./out.txt '{"agent":{"name":"echo"},"path":["workspace","out.txt"]}'
diverge run agents download --path ./ws      '{"agent":{"name":"echo"},"path":["workspace"]}'

# upload one file; --path is the local file
diverge run agents upload --path ./in.txt '{"kind":"file","agent":{"name":"echo"},"path":["workspace","in.txt"]}'

# upload a directory; --path is the local directory, "files" names each file beneath it
diverge run agents upload --path ./src '{"kind":"directory","agent":{"name":"echo"},"path":["workspace","src"],"files":["main.py","lib/util.py"]}'

# copy container → agent | tool | dependency tool | volume, without the bytes reaching you
diverge run agents transfer '{"agent":{"name":"echo"},"path":["workspace"],"to":{"volume":{"provider":{"issuer":"…","subject":"ronald.provider"},"name":"notes"},"path":["backup"]}}'
```

- A container that is not running is **started for the operation and stopped after it**, so these read the
  image plus the mounts, not the previous run's own disk. That start deploys every tool the agent is served
  and fails, with the operation, if any one cannot be deployed.
- `filetree` shows the whole container including FUSE mounts; one snapshot line, then changes under `--watch`.
- `download` prints one line per file written. `"not_found"` is no such agent or nothing at the path. A
  directory with no files prints nothing.
- `upload` replaces each file whole and makes parents. A directory's `files` are `/`-separated paths from the
  directory's root; at least one, none twice. Answers `"uploaded"`, `"not_found"`.
- `transfer` destinations: `{"agent":<ref>,"path":[…]}`, `{"tool":<ref>,"path":[…]}`,
  `{"dependency":{"template":…,"owner":…},"path":[…]}`, `{"volume":{"provider":…,"name":…},"path":[…]}`.
  Answers `"transferred"`, `"not_found"`, `"no_destination"`, `"held"` (a volume at either end is in use by a
  running container or another file operation). Existing files at the destination that the source lacks stay.

## exec, terminal, frontend

```bash
# one command, no shell in between; JSON lines by default
diverge run agents exec '{"agent":{"name":"echo"},"argv":["ls","-la"],"cwd":["workspace"],"env":{"LANG":"C"}}'
# {"stdout":"<base64>"} … {"exited":0}

# the command's own output and exit code
diverge run agents exec --raw '{"agent":{"name":"echo"},"argv":["sh","-c","cat /etc/os-release | head -3"]}'

# an interactive shell: stdin goes to it, its output to stdout, exit code is the shell's
diverge run agents terminal '{"agent":{"name":"echo"},"cols":120,"rows":40}'
printf 'ls /\nexit\n' | diverge run agents terminal '{"agent":{"name":"echo"},"cols":120,"rows":40}'

# one HTTP request to the page the agent's program serves; body is stdin
diverge run agents frontend '{"agent":{"name":"me"},"method":"GET","path":"/frontend/","headers":[]}' </dev/null
```

- `exec`: `argv` is required and run as is (use `["sh","-c","…"]` for pipes). `cwd` and `env` are optional.
  Without `--raw`, stdout and stderr arrive base64-encoded, ending with `{"exited":<code>}` or
  `{"signaled":<signal>}`. Ending the command kills the one in the container.
- `terminal`: `cols` and `rows` are required. When stdin ends the shell receives Ctrl-D. Piped use is
  possible but output is a terminal's (escape codes). Prefer `exec` for scripted work.
- `frontend`: only paths beginning `/frontend` are reachable. `headers` is a list of `[name, value]` pairs.
  The response head is one JSON line on **stderr**, the body raw on **stdout**; exit `0` for a whole
  response. `"no_frontend"` means the image declares none (its `DIVERGE.md` has no `"frontend": true`).
- All three start a stopped container and hold it up while they run.
- `exec` and `terminal` both need the `exec` grant; `frontend` needs `frontend`.

## An agent's permissions

An agent's program may put requests to the daemon over its own daemon connection, as a tool's may. Each time
its container starts, the program declares what it may ask: a list of grants, in the vocabulary of the
`diverge-access` grants reference. An agent may declare any grant a tool may. For every kind a grant reaches
`"any"`, exactly the records it lists, or by tags, and never two of those in one grant.

Over the three tool kinds (`standalone_tools`, `connected_standalone_tools`, `dependency_tools`) there is
one word more, `"attached"`: the tools served to this agent, which are its dependency tools and the
standalone and connected tools attached to it or to its template. `"attached"` over agents reaches nothing
for an agent: that direction is a tool's (see `diverge-tools`).

- You do not set these; the image does. They are reported as `permissions` on the agent's list item after
  its first start (absent before, or when it declares none).
- The daemon holds the agent to exactly those. An action held only within `"attached"` is held at all only
  while a tool of that kind is attached.
- An agent has no account, so `frontend` and `standalone-tools connect` refuse it whatever it declares.
- A scope the program has open on its daemon connection holds the container up, without making the agent
  active. A program that wants its container to go idle closes its streams.
- Every request the agent's container puts to the daemon is kept in the agent's own log as a `command`
  item: the request's path and body and `allowed` (`false` exactly when the daemon answered `forbidden`).
- An agent can also be asked: a request's `to` may name agents, and an agent's program listens for the
  requests addressed to it. See `diverge-access`.

## Grants

Each leaf needs a grant of the same name over agents: `create` (making), and over existing agents `get`,
`list`, `edit`, `delete`, `message`, `logs`, `download`, `upload`, `transfer`, `filetree`, `exec` (also
`terminal`), `frontend`, `connect`, plus `tag`/`untag`. Templates: `create`, `get`, `list`, `arguments`, `delete`,
`attach`, `tag`/`untag` over `agents_templates`. A create or edit that names a volume also needs `mount` over
the volume; a transfer needs `transfer` over the source
and `upload` over the destination. `"forbidden"` means no grant reaches what the request names, and nothing
changed. A list is never partially forbidden: it sends only what the account's `list` grants reach. See
`diverge-access`.

## Reference files

- [references/mounts.md](references/mounts.md) — provider pin, volume mounts, FUSE file and directory mounts,
  volume modes, edit patches for mounts.
- [references/answers.md](references/answers.md) — every leaf with its body and every answer it can print.

## Not verified

- The exact JSON of the line `download` prints per file written (it carries the file's path and byte count;
  run `agents download response-schema`).
- That a stopped container's own disk is discarded (so only mounts, the agent's and the tools' databases and the log carry
  state between runs). This follows from the template and create documentation ("its state is its
  conversation … and nothing on any provider's disk") but was not tested.
- `agents frontend` and `agents terminal` were not run live while writing this; behaviour is from the CLI and
  SDK source.
- The installed binary's `agents message response-schema` may still describe a cancel; the source has none,
  and the rules above follow the source.

## When this task is done

Load the `diverge` skill again before starting the next Diverge task: it is the index, and it says which
skill the next task needs.
