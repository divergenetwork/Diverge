# The example tools

Seven images under `diverge-examples/` in the repository. Six are tools; `human` is an agent image listed here because
it is used with them. Each image's `/.diverge/DIVERGE.md` states its type, its name, its one-line description, its arguments schema, the memory and disk it needs, for a tool its `mode` (and for a dependency tool its `scope`), and for a tool whether it serves
a frontend or an API; read it from a running daemon with `diverge run images read --info` (`diverge-images`), which is also
where you get the digest to put in a template. All take `arguments: {}` (any member is refused). Each tool image states `"memory":268435456,"disk":268435456`.

An image is exactly one kind. `memory`, `taskboard`, `courier` and `alarm` are `dependency_tool` images: an
agent's program declares them, and a tool template of one is refused when a tool is created from it. `chat`
is a `standalone_tool` image: it is made from a tool template, and an agent that declares it as a dependency
is refused.

In every example below `<digest>` is the image's digest in the provider's store, and the `references` entry
must name a registry the provider lists (for locally built images that is typically `docker.io` with the
local repository name; nothing is pulled when the provider already holds the digest).

## memory — a notebook (dependency, lazy, instance)

| MCP tool | parameters | does |
|---|---|---|
| `remember` | `text`, `tags` (optional) | keeps a note; answers its id |
| `recall` | `query` (optional), `tags` (optional), `limit` (default 20) | notes carrying every tag given; with `query`, a text search, best match first; otherwise newest first |
| `forget` | `id` | deletes a note |

- Permissions: none. The notes are in the tool's `instance` database, which it reaches through its
  container's proxy, not through the daemon.
- The tool records no owner. Its document says `"mode":"lazy","scope":"instance"` and it keeps its notes
  in `instance`, which is the declaring agent's own and persists between the agent's runs.

Declare it in an agent template (python agent-loop image shown; `openrouter`, `cc` and the others take the
same `dependencies` member):

```bash
diverge run agents templates create '{
  "type": "agent",
  "image": {"digest": "<python digest>", "references": [{"registry": "docker.io", "name": "diverge-local/python"}]},
  "extra_memory": 0, "extra_disk": 0,
  "arguments": {
    "python": "…agent script…",
    "dependencies": [
      {"image": {"digest": "<memory digest>", "references": [{"registry": "docker.io", "name": "diverge-local/memory"}]},
       "extra_memory": 0, "extra_disk": 0, "arguments": {}}
    ]
  }
}'
# → {"created":"<agent template id>"}; then agents create, agents message
diverge run agents connect list-tools '{"agent":{"name":"echo"}}'      # <8 hex>_remember, <8 hex>_recall, <8 hex>_forget
diverge run dependency-tools list '{"arguments":false,"owners":[{"agent":{"name":"echo"}}]}'
```

## taskboard — a shared board (dependency, eager, template)

| MCP tool | parameters | does |
|---|---|---|
| `create_task` | `title`, `detail` | posts an open task; answers its id |
| `list_tasks` | `state` (`open`, `claimed`, `done`; optional), `limit` (default 50) | the board's tasks, oldest first |
| `claim_task` | `task` | claims an open task for the caller (atomic: two claimers never both win) |
| `complete_task` | `task`, `result` | completes a task the caller holds |
| `release_task` | `task` | puts a task the caller holds back to open |
| `delete_task` | `task` | deletes a task the caller posted, with its comments |
| `create_comment` | `task`, `text` | comments on a task |
| `list_comments` | `task` | a task's comments, oldest first |

- Permissions: none. Pull only: nothing is sent to an agent when the board changes.
- Callers are recorded as the daemon attests them. Only agents may use it.
- Its document says `"mode":"eager","scope":"template"`: one container for every agent that declares it.
  The board is in the tool's `template` database, so every agent that declares the byte-identical
  template works from one board, and a different declaration is a different board.

## courier — an inbox between agents (dependency, eager, template)

| MCP tool | parameters | does |
|---|---|---|
| `list_agents` | none | agents the caller can write to: those using this courier, then those that have never run |
| `create_conversation` | `to` (`{"name":…}` or `{"template":…,"index":n}`), `topic`, `text` | opens a conversation with another agent, with its first message |
| `reply` | `conversation`, `text` | adds a message to one of the caller's conversations |
| `list_conversations` | `include_read` (default false) | the caller's conversations with unread messages, newest first |
| `list_messages` | `conversation`, `include_read` (default false) | a conversation's unread messages, oldest first, and marks them read |

- On each send the courier messages the recipient agent exactly
  `<courier>\nYou have a new message\n</courier>`; the daemon starts the recipient if it is not running. The
  mail itself stays in the courier's database, read through the tools.
- Permissions it declares for itself:

  ```json
  [
    {"agents": {"actions": ["get", "list", "message"], "within": "attached"}},
    {"agents": {"actions": ["get", "list", "message"], "within": "never_active"}}
  ]
  ```

  `attached` is every agent whose last registration declared this courier; `never_active` is every agent
  that has never run, so one can be written to before its first start. An agent that has run and does not
  declare the courier is unreachable, and an agent whose image never declares it can be written to but can
  never read its mail.
- Its document says `"mode":"eager","scope":"template"`. Declare it in **every** participating agent's template, with the
  byte-identical template: the dependency template's id is what the mail is kept under, so any difference
  (arguments, `extra_memory`, `extra_disk`, digest) is a separate mail system.
- Only agents may use it.

## alarm — wake me later (dependency, eager, template)

| MCP tool | parameters | does |
|---|---|---|
| `set_alarm` | `note`, and exactly one of `at` (RFC 3339), `in_seconds`, `cron` (five fields, UTC) | sets an alarm for the caller; answers its id and when it is first due |
| `list_alarms` | none | the caller's alarms, soonest first |
| `cancel_alarm` | `alarm` | cancels one of the caller's alarms |

- When due, the tool messages the agent that set it exactly `<alarm>\n{note}\n</alarm>`; the daemon starts
  the agent if it is idle.
- Permissions it declares: `[{"agents":{"actions":["message"],"within":"attached"}}]`.
- Its document says `"mode":"eager"` (a lazy tool stops when nothing uses it and would keep no time) and
  `"scope":"template"`. Each alarm wakes only the agent that set it.
- An alarm that cannot be sent when due is dropped. Alarms that fell due while the tool was down fire once
  when it returns; missed repeats of a cron alarm are skipped.
- Only agents may use it.

## chat — a person talks to agents (standalone tool, eager, frontend)

| MCP tool | parameters | does |
|---|---|---|
| `read_messages` | `include_read` (default false) | the person's messages to the calling agent, oldest first; unread ones are marked read |
| `send_message` | `text` | sends the person a message, shown in every open page |

- The person writes in the tool's frontend (one tab per attached agent). On each message the tool messages
  the agent exactly `<chat>\nYou have a new message\n</chat>`; the agent then calls `read_messages`.
- Its `DIVERGE.md` declares `"type": "standalone_tool"`, `"mode": "eager"` and `"frontend": true`.
- Permissions it declares: `get`, `list` and `message` over the agents attached to it, `get` over agent
  templates, `read` over providers and `relay` over remote daemons.

```bash
TPL="$(diverge run standalone-tools templates create --jq '.created // .exists' '{
  "type": "tool",
  "image": {"digest": "<chat digest>", "references": [{"registry": "docker.io", "name": "diverge-local/chat"}]},
  "extra_memory": 0, "extra_disk": 0, "arguments": {}
}' | tr -d '"')"
diverge run standalone-tools create "{\"template\":\"$TPL\",\"name\":\"chat\"}"
diverge run standalone-tools attach-agent-instance '{"tool":{"name":"chat"},"agent":{"name":"echo"}}'
diverge run standalone-tools get '{"tool":{"name":"chat"}}'            # deployed: true once the eager tool is up
diverge run standalone-tools frontend '{"tool":{"name":"chat"},"method":"GET","path":"/frontend/","headers":[]}' </dev/null
diverge run agents connect list-tools '{"agent":{"name":"echo"}}'      # chat_read_messages, chat_send_message
```

The page is meant for a browser; the viewer (diverge-viewer) opens a tool's frontend tab. The CLI's `frontend`
leaf is one HTTP request at a time and is enough to check that the page is served.

Taking it down again:

```bash
diverge run standalone-tools detach-agent-instance '{"tool":{"name":"chat"},"agent":{"name":"echo"}}'
diverge run standalone-tools delete '{"tool":{"name":"chat"}}'
diverge run standalone-tools templates delete "{\"id\":\"$TPL\"}"
```

## human and human-desk — an agent with no model (agent image, and its dependency tool)

`human` is not a tool. It is an agent image whose "model" is a person, and it has no page of its own, since
an agent has no frontend. It declares one dependency tool, `human-desk` (`"mode": "eager"`,
`"scope": "instance"`, `"frontend": true`, `"api": true`), and the grant `api` over its attached dependency
tools.

- A message waits at the desk until someone answers there: writing the agent's replies, calling its
  attached tools by hand, and ending the turn. Nothing times out.
- The agent reaches the desk over `api`; the person reaches it over `frontend`.
- Make `human` like any agent (`agents templates create` with `arguments: {}`, `agents create`) and attach
  the tools the person should be able to call. The desk is made by the agent's registration.
- Open the desk's frontend, not the agent's: the viewer's frontend tab on the dependency tool, or

```bash
diverge run dependency-tools frontend '{"dependency":{"template":"<desk template id>","owner":{"agent":{"name":"me"}}},"method":"GET","path":"/frontend/","headers":[]}' </dev/null
```

  The desk's template id is the agent's `dependencies` entry on `agents get`.

It is the
quickest way to exercise a tool exactly as an agent would without a model: so is
`diverge run agents connect call-tool` against any agent the tool is attached to.

## Which shape for which need

| need | use |
|---|---|
| private state for one agent that survives its restarts | a dependency tool image that says `lazy`, `scope: "instance"`, keeping its state in its `instance` database (memory) |
| state shared by every agent that declares the same dependency template | any dependency tool image keeping its state in its `template` database (taskboard, courier); `scope: "template"` if one container should serve them all |
| state shared by every tool of an image, whatever its arguments | its `image` database |
| something that must keep running between agent runs | an image that says `eager`, dependency or standalone (alarm, chat) |
| a tool you pick per agent at runtime, or that has a page for people | a standalone tool, attached (chat) |
| a tool that lives on another daemon | a connected standalone tool (`diverge-daemons`) |
