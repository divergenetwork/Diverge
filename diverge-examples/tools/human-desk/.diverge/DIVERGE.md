---
{
  "type": "dependency_tool",
  "name": "human-desk",
  "description": "The desk a person answers at as the human agent: its page, and the agent's stream.",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments a human desk is made with.\n\nNone: the desk is its one agent's, and the person at it is whoever\nopens its frontend. `{}` is the one value; any member is refused,\nso a misspelled setting is never silently ignored.\n\n# How it runs\n\nAs a dependency tool of the `human` agent: an eager tool whose\nscope is `instance`, so each agent has a desk of its own, kept up\nwhile the agent declares it, with the page's sockets held between\nthe agent's turns.",
    "type": "object",
    "additionalProperties": false
  },
  "memory": 268435456,
  "disk": 268435456,
  "mode": "eager",
  "scope": "instance",
  "frontend": true,
  "api": true
}
---
# Human desk

The desk of a **`human`** agent: where a person answers as that agent.
The agent has no model and no page; this tool is its page. It is the
agent's own dependency tool, one desk per agent.

- **The frontend** shows the whole transcript, live: every message that
  came in, every reply, every tool call with its result. A message that
  arrives while you are at it is shown at once and is part of the same
  turn.
- **Replies** are the agent's text; **Call** has the agent run one of
  its tools with the JSON arguments you give, prefilled from the tool's
  schema; **End turn** ends the agent's run.
- **The tools are the agent's.** The desk holds none: it asks the agent
  to list them and to make each call, and shows what comes back. They
  are listed while a message is waiting on you.
- **Nothing times out.** The agent waits on you for as long as it takes.
- **The transcript** is kept in the desk's `instance` database, which is
  this agent's desk's and no other's, so it survives the container
  starting again.

Arguments: `{}`.

## Its two doors

| door | who may come through | what it is |
|---|---|---|
| `/frontend` | a person: a request the daemon stamps as a client | the page, and one live socket per open page |
| `/api/desk` | the agent the desk is deployed for, and nobody else | the agent's stream, one for each turn |

A request through either door that the daemon stamps as anyone else is
refused. The desk serves an MCP server, as every tool does, with no
tool on it. It declares no permission: it asks the daemon nothing.

## The agent's stream

The agent opens `GET /api/desk` with `Connection: upgrade` and
`Upgrade: human-desk`, and the desk answers `101`. From then each side
sends lines, one JSON value each:

| from the agent | |
|---|---|
| `{"type":"messages","messages":[{"key":…,"content":[…]}]}` | the run's messages, first; then every batch enqueued while the turn is open |
| `{"type":"tools","tools":[…]}` | the agent's tools, listed; with `error` where they could not be |
| `{"type":"result","id":…,"result":{…}}` | what a tool call came to |

| from the desk | |
|---|---|
| `{"type":"chunk","chunk":{…}}` | a chunk recorded on the turn, which the agent's run emits |
| `{"type":"call","id":…,"name":…,"arguments":{…}}` | a tool call for the agent to make |
| `{"type":"tools"}` | list the tools again |
| `{"type":"end"}` | the person ended the turn; the desk closes its side |

## Running it

Nothing is made by hand: the `human` agent declares this image as its
dependency, and the daemon deploys a desk for each such agent. Give the
`human` agent's template this image's digest as its `desk` argument,
message the agent, and open this tool's **frontend**.
