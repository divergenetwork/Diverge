---
name: diverge
description: The index and core of the Diverge skills. Load this FIRST for any task that touches Diverge — the `diverge` CLI, a Diverge daemon, provider, identities, agents, agent templates, standalone / dependency / connected tools, images, volumes, the vault, accounts, roles, grants, events, logs, remote daemons, relays, or the diverge-viewer viewer — and load it AGAIN after each such task finishes. It explains the concepts, the shape of the CLI (`diverge run <family> <request> [BODY]`, `request-schema`, `response-schema`, `--jq`, `--watch`), the rules that keep an agent from hurting itself, and which one of the other Diverge skills to load for the task at hand.
---

# Diverge: index and core

Diverge runs AI agents and their tools as containers. A **daemon** holds every record (agents, tools, accounts,
secrets, logs) and is the only thing a client talks to; a **provider** runs the containers for it; an **issuer**
says who everyone is. You operate all of it from outside, through one program: the `diverge` CLI.

## Protocol: how to use the Diverge skills

Follow this for every Diverge task.

1. **Load this skill first.** Do not run a `diverge` command for a new task before reading it.
2. **Load exactly one sub-skill**: the one the table in "Which skill to load" names for the task. Do the task
   from that skill.
3. **When the task is done, load `diverge` again** before starting the next Diverge task, even if the next task
   looks similar. The next task may belong to a different skill, and this index is how you find out.

If one user request spans several skills (for example "make a volume, then an agent that mounts it"), split it
into tasks and run the protocol once per task.

## Concepts

| Term | What it is |
|---|---|
| Config root | The one directory the Diverge programs on a host keep their settings and signed-in identities in. `--config <dir>`, else `DIVERGE_CONFIG`, else `~/.diverge`. One of each program per root. **Never look inside it or change anything in it by hand**: every setting has a `diverge` command (skill `diverge-config`), and it holds secrets. |
| Issuer | The OpenID Connect provider that signs the tokens an identity proves itself with. An issuer can be used only if it supports DPoP (RFC 9449) with ES256, answers access tokens that are signed JWTs carrying `sub` and the key's thumbprint (`cnf.jkt`), and offers the device authorization flow. Diverge's own is `https://identity.diverge.network`, hosted, whose subjects are `<username>.<name>` (for example `ronald.daemon-a`): the username a GitHub or Google account set there, once and for good, and a name that account chose. A login there needs no person once that account is signed in on the host and has its username. |
| Identity | One `(issuer, subject)` pair signed in under a root, with a private key kept on that host. Every handshake is mutual: both ends present an access token bound to their key, send the key, and prove they hold it. A copied token is useless without the key, and every connection is encrypted after the handshake. |
| Daemon | `diverge-daemon` (WebSocket, default port 14980). The hub: keeps accounts, roles, provider records, templates, agents, tools, vault, events and logs in its own Postgres, and starts and stops containers on providers. |
| Account | An identity the daemon admits as a client, holding **roles**; each role is a list of **grants**. Identities in the daemon's `admins` setting are admitted without an account and hold every grant. |
| Provider | `diverge-provider` (default port 14979). Runs containers with podman for the identities it accepts, when containers are enabled on it; also serves volumes and an image store, each when enabled. A fresh provider has none of the three. A daemon holds a *record* of each provider it uses. |
| Container proxy | A small binary the provider mounts into every container. It carries messages, MCP, Postgres and secrets between the provider and the program inside. Never part of an image; never operated directly. |
| Image | An OCI image, named by digest plus registry references, whose program speaks the container contract. It carries a `DIVERGE.md` stating its type (`agent`, `standalone_tool` or `dependency_tool`), its name, a one-line description of what it is, the schema of the arguments it takes, the memory and disk it needs, every secret its program could take, and, for a tool, how it is kept running. |
| Template | An image plus the memory and disk it gets beyond what the image needs, arguments and the names of the secrets it needs, each one the image states. Its id is a hash of its content: creating the same template twice answers the same id. There are agent templates and standalone-tool templates. |
| Agent | One container made from an agent template. It starts when it is messaged, is stopped after `idle_seconds` of disuse (default 10), and starts again on the next message. Its record and log persist. |
| Standalone tool | An MCP-server container made by hand from a tool template of a `standalone_tool` image and attached to agents (or agent templates). |
| Dependency tool | A tool of a `dependency_tool` image that an agent's own arguments declare (`dependencies`); the daemon deploys it for that agent, its template, or everyone, as the tool's image says. |
| Connected standalone tool | Another daemon's standalone tool, held on this daemon under a name so this daemon's agents can use it. |
| Volume | Storage a provider holds, mounted into containers and readable and writable as files through the daemon. |
| Vault | The daemon's key-value store of secrets. A template names the secrets it needs; a create binds each name to a vault key. Secrets are never arguments. |
| Events | The daemon's graph of who sent what to whom (accounts, agents, tools). |
| Remote daemon | A record on this daemon of another daemon, reached through a provider both are connected to. Its id is a short base62 hash. |
| Relay | `"relay": [<daemon id>, …]` in any request body: this daemon forwards the request to that remote daemon, which serves it as the account the record names. |
| Viewer | `diverge-viewer`, a desktop app that draws one daemon's board (providers, accounts, agents, tools, events). It keeps its own list of daemons and accounts, as the CLI does (`diverge viewer daemons …`, `diverge viewer default …`), and is started on one of them and stopped with `diverge viewer start [--daemon <name>] [--account <name>]` and `diverge viewer stop`; its window's title names the daemon. One per root. |

## The shape of the CLI

```
diverge [--config <dir>] <command> …
```

Top-level commands: `run`, `identities`, `daemon`, `provider`, `viewer`, `cli` (and `help`).
Everything except `run` works on the local root and is covered by `diverge-config`.

```
diverge [--config <dir>] run [--daemon <name>] [--account <name>] [--jq <filter>] [--timeout <seconds>] \
    <family> [<family>…] <request> [BODY]
```

- A **family** is a group of requests: `agents`, `standalone-tools`, `volumes`… Some nest: `agents templates
  create`, `providers daemons list`, `agents connect call-tool`. `info` and `logs` are requests with no family.
- **BODY** is one JSON object, as one shell argument. Absent, it is `{}`. Single-quote it in the shell.
- `--daemon` and `--account` pick a daemon and an account by the names the CLI has them under; absent, the
  CLI's defaults. They, `--jq` and `--timeout` may be written anywhere after `run`.
- Some requests take flags of their own: `--watch` on every `list` but `identities list`, and on the agent and
  tool `filetree`s (keep the
  stream open and print every change after the first whole answer; without it the command ends there),
  `--path <local>` on the file `download`/`upload` requests, and typed flags on `connect`, `exec`, `terminal`,
  `frontend`. Read `<request> --help`.

**Discovering a request.** These dial nothing and are always safe:

```bash
diverge run --help                                  # the families
diverge run agents --help                           # the requests of a family
diverge run agents create --help                    # its flags
diverge run agents create request-schema            # JSON Schema of the BODY
diverge run agents create response-schema           # JSON Schema of every line it can print
diverge run agents create request-schema --jq '.properties | keys'
```

Run `request-schema` before writing a body you have not written before. Unknown members are refused, with the
path to what was wrong.

**How answers are printed.** One compact JSON line per answer frame, on stdout, each printed the moment it
arrives. A frame is a snake_case word or a one-member object: `"delivered"`, `{"created":"…"}`, `"not_found"`,
`"forbidden"`, `{"error":…}`. A list prints one line per item and then `"listed"`. With `--jq`, each frame goes
through the filter: every value it yields is a line, and `null` or nothing prints nothing — so
`--jq '.created // .exists'` prints just the id.

**Exit codes.**

| Code | Meaning |
|---|---|
| 0 | The request was put and its answers printed. **`"forbidden"`, `"not_found"` and `{"error":…}` are answers: they exit 0.** Read the line, not the code. |
| 1 | The CLI itself failed; one line `Error: …` on stderr. Bad body, unknown daemon or account name, daemon not reachable, handshake failed, `--timeout` passed, a document refused. |
| 2 | The command line did not parse (unknown subcommand, missing flag). |
| other | `exec` and `terminal` requests exit with the code of the command they ran. |

**Root resolution.** The CLI finds its root in this order: `--config <dir>`; else the `DIVERGE_CONFIG`
environment variable; else `~/.diverge`. The daemon, provider and viewer do the same. A `.diverge` folder in
the working directory is not a root: it is where an image's source keeps its `DIVERGE.md`.

## Rules that keep you from hurting yourself

0. **Never look inside the config root, and never change anything in it by hand.** Do not open, list, read,
   search or edit what is in `~/.diverge` (or wherever `--config` or `DIVERGE_CONFIG` points). It holds the
   signed-in identities and their private keys, which are secrets, and nothing in it is yours to edit. Every
   setting is read with a `get` and changed with a `diverge` command (skill `diverge-config`). If no command
   does what you need, tell the user.
1. **Never pipe `diverge … start`.** `diverge daemon|provider start` leaves a detached server running;
   a pipe (`| tail`, `| tee`, `$(…)`) is inherited by it and your shell hangs. Redirect to a file and read the
   file: `diverge daemon start > start.out 2>&1; cat start.out`.
2. **Do not start, stop or reconfigure a server unless the task asks for it.** A provider stop removes every
   container it ran; a daemon stop ends every agent's run.
3. **`--config` goes first**, before the command: `diverge --config <dir> daemon start`. It is not accepted
   after the command. `--daemon`, `--account`, `--jq`, `--timeout` belong to `run` and go after it.
4. **Check which root you are on.** `DIVERGE_CONFIG`, where it is set, silently names the root.
   `diverge cli daemons get`, then `diverge cli daemons <INDEX> accounts get`, show the daemons and accounts the
   root you are on knows.
5. **A `run` needs an account.** With no `--account` and no default account for the daemon in the CLI's settings, every
   `run` command fails — even `request-schema` — with `no account was named, and the daemon … has no default`.
   Add an account and make it the default: `diverge cli daemons <INDEX> accounts add …`, then
   `diverge cli daemons <INDEX> default set <INDEX>` (skill `diverge-config`).
6. **`relay` exists on every request.** Any body may carry `"relay": ["<daemon id>"]` to have a remote daemon
   serve the request instead. Leave it out unless you mean it (skill `diverge-daemons`).
7. **Naming an agent or a tool: by name, or by template and index.** `{"name":"echo"}` names the one that has
   the name now; the name is free again after a delete. `{"template":"<id>","index":<n>}` names one for ever,
   and is the only way to reach an agent or tool created without a name. Never put members of both in one
   object. A connected tool is `{"name":…}` or `{"daemon":…,"tool":…,"index":…}`; a dependency tool is
   `{"template":…,"owner":…}`; a volume is `{"provider":…,"name":…}`.
8. **What `"forbidden"` means.** The daemon admitted you, but the account holds no grant allowing this action on
   this thing (or may not use the relay named). Nothing was done. It is not a transient failure: do not retry;
   diagnose it with the `diverge-access` skill. This differs from `Error: the daemon closed the connection
   without answering`, which means the identity is not an account or an admin of that daemon at all.
9. **`"not_found"` is an answer about one daemon and one account.** Before concluding a thing is gone, check
   the reference (rule 7), `--daemon`, and `--account`.
10. **Deletes are final.** A deleted agent, tool, volume, vault key or account is gone, with its data. Confirm
    with the user before a delete they did not ask for.
11. **Never print or store secrets.** Vault values and tokens go in on stdin or in a body; no command
    prints a token, and you should not echo one either.
12. **Lists end; watches do not.** A command with `--watch`, or a `logs` body with `"watch": true`, runs until
    you end it. Give it `--timeout` or run it in the background.

## Which skill to load

Every `diverge run` family and every top-level command belongs to exactly one skill.

| Task | Skill | Commands it owns |
|---|---|---|
| The config root; every setting of the daemon, the provider, the viewer and the CLI; signing identities in; the accounts at Diverge's own issuer; starting and stopping a server; ports; what needs a restart; the identities on the daemon's host | `diverge-config` | `diverge daemon start\|stop` and its settings (`port`, `idle_seconds`, `admins`, `identity`, `postgres`), `diverge provider start\|stop` and its settings (`port`, `identity`, `incoming`, `outgoing`, `podman`, `containers`, `volumes`, `images`, `messages`), `diverge viewer start\|stop` and its settings (`port`, `redial`, `daemons`, `default`), `diverge cli daemons …` and `diverge cli default`, `diverge identity.diverge.network accounts\|subjects\|refresh-tokens …` (with `accounts username get\|set`), `diverge identities login\|list\|remove`, `diverge run identities list\|tag\|untag\|login` |
| Agent templates and agents: create, get, list, edit, delete, tag; sending a message; MCP into an agent; an agent's files, exec, terminal, frontend, mounts | `diverge-agents` | `diverge run agents …` (including `agents templates …` and `agents connect …`) |
| Standalone tools and their templates; dependency tools; attaching and detaching tools; a tool's files, exec, terminal, frontend; MCP exchanges with a tool | `diverge-tools` | `diverge run standalone-tools …` (including `templates …` and `connect …`), `diverge run dependency-tools …` |
| Images: an image's `DIVERGE.md`, digests and references, building images into the provider, what each stock image takes as arguments and secrets | `diverge-images` | `diverge run images read\|build\|push\|merge\|publish\|list\|get\|delete` |
| Volumes and their files; the vault and binding secrets; the daemon's Postgres and the compartments of it each container reaches | `diverge-storage` | `diverge run volumes …`, `diverge run vault …`, `diverge run postgres …` (including `postgres compartments …`) |
| Accounts, roles, grants, tags and `within`, admins; why something is `forbidden`; asking accounts and tools a question and answering the ones asked of you | `diverge-access` | `diverge run accounts …`, `diverge run roles …`, `diverge run requests create\|listen` |
| Provider records on the daemon; other daemons visible through a provider and messaging them; remote daemon records; `relay`; tools held from another daemon | `diverge-daemons` | `diverge run providers …` (including `providers daemons list\|listen\|message`), `diverge run daemons …`, `diverge run connected-standalone-tools …` |
| Reading what happened: logs of an agent, tool or account; the event graph; watching lists; the daemon's version; shaping output with `--jq` | `diverge-logs` | `diverge run logs`, `diverge run events list`, `diverge run info` |

Choosing when a task seems to fit two:

- The *grant* that allows an action is `diverge-access`, whatever the action is on.
- A *secret's value* is `diverge-storage`; *which secrets an image needs* is `diverge-images`; *binding* them at
  create is in `diverge-agents` / `diverge-tools`.
- A tool's log is read with `diverge run logs` (`diverge-logs`); the tool itself is `diverge-tools`.
- A tool on *another* daemon is `diverge-daemons`, even though it is attached to an agent.
- "Is it running / reachable / which version" is `diverge-logs` (`diverge run info`); "start it" is
  `diverge-config`.
- Installing Diverge from source is not a skill: it is `reports/SETUP_INSTRUCTIONS_1.md` in the repository.

## A first look at an installation

Read-only, safe on any running installation:

```bash
diverge cli daemons get                     # which daemons this root names (then: diverge cli daemons 0 accounts get)
diverge identities list                     # which identities are signed in here
diverge run info                            # {"info":{"version":"…"}} — the daemon is up and admits you
diverge run providers list                  # each provider record; "status" should be "connected"
diverge run agents list                     # one line per agent, then "listed"
```

If `diverge run info` fails with `the daemon could not be dialled`, the daemon is down or the CLI has the
wrong address for it; with `the handshake did not complete`, the account's identity is not signed in (or is signed
out) under this root; with `the daemon closed the connection without answering`, the identity is neither an
account nor an admin there. All three are `diverge-config` tasks.

## Now

Pick the one skill the table names for the task and load it. When the task is done, load `diverge` again.
