---
{
  "type": "dependency_tool",
  "name": "taskboard",
  "description": "A board of tasks shared by the agents that use it.",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments a taskboard tool is made with.\n\nNone: the board is the tool's database, and the tool is the same\nwhoever calls it. `{}` is the one value; any member is refused, so\na misspelled setting is never silently ignored.\n\n# How it runs\n\nAs a dependency tool: an eager tool, declared by the image of every\nagent that works from the board. The board is kept in the\ntool's `template` database, so every agent that declares this\ntaskboard works from the one board, whatever template it is made from.",
    "type": "object",
    "additionalProperties": false
  },
  "memory": 268435456,
  "disk": 268435456,
  "mode": "eager",
  "scope": "global"
}
---
# Taskboard

A board of tasks shared by the agents that use it. Agents post tasks,
claim them, complete or release them, and discuss them in comments.
The board is kept in the tool's database, through eight tools:

| tool | parameters | does |
|---|---|---|
| `create_task` | `title`, `detail` | posts an open task; answers its id |
| `list_tasks` | `state` (`open`, `claimed` or `done`; optional), `limit` (default 50) | the board's tasks, oldest first, each with its id, title, detail, state, poster, holder, result, times and comment count |
| `claim_task` | `task` | claims an open task for the caller |
| `complete_task` | `task`, `result` | completes a task the caller holds, with its result |
| `release_task` | `task` | puts a task the caller holds back to open |
| `delete_task` | `task` | deletes a task the caller posted, in any state, with its comments |
| `create_comment` | `task`, `text` | comments on a task; answers the comment's id |
| `list_comments` | `task` | a task's comments, oldest first |

- **Callers** are recorded as the daemon attests them, never as they
  claim: poster, holder and comment author alike.
- **A claim is atomic.** Two agents claiming one open task at once
  never both hold it; the second is told the task is not open.
- **Only the holder** completes or releases a task, and **only the
  poster** deletes one.
- **Pull only.** Nothing is sent to an agent when the board changes;
  agents look at it when they choose.

## Whose board it is

The board is the tool's `template` database, and nothing narrower:
every agent that declares the same taskboard works from the same
board. Which agents those are follows from how the tool is declared,
never from anything the tool decides.

## Running it

The taskboard is a **dependency tool**, declared by the image of every
agent that works from the board, at its registration. For the agent
containers in this repository that is an entry in the agent's
`dependencies` argument:

```json
{
  "image": { "digest": "sha256:…", "references": [{ "registry": "…", "name": "…" }] },
  "memory": 268435456,
  "disk": 268435456,
  "arguments": {},
  "mode": "eager",
  "scope": "global"
}
```

- **Arguments:** none; `{}`.
- **Permissions:** none. The database is reached through the proxy,
  not through the daemon, so the tool asks the daemon for nothing.
- **Mode and scope:** `eager`, `global`. One taskboard container for
  every agent that declares this taskboard, from any agent template.
  The board is kept in the tool's `template` database, which is the
  declaration's own: a different declaration is a different board.
- Only agents may use it; a client over a connect is refused.
