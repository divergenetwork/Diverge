# The agent-loop images: python, lone, openrouter, codex, cc

Reference for five of the six agent-loop images (hermes is in [hermes.md](hermes.md)). For each:
what it is, its `arguments`, which of them carries its credential, its dependencies, and
a minimal working **agent template body** for `diverge run agents templates create`.

Conventions used below:

- `<digest>` is the image's manifest digest, as `diverge run images build` printed it in its last
  line (`{"built":{"digest":…}}`).
- Every body carries a `references` entry naming a registry the provider lists (`docker.io` by
  default), which the daemon requires even for a locally built image.
- `extra_memory` and `extra_disk` are **bytes** and are required on every template: what the container
  gets beyond the `memory` and `disk` its image's `DIVERGE.md` states. The container's limit is the sum.
  Each of these five images states `"memory":536870912,"disk":1073741824`, so `0` gives a container
  512 MiB of memory and 1 GiB of disk. The extras shown are starting points, not measured minimums.
- A credential is a member of `arguments`. Whoever may see the template's arguments sees it, so read a
  template with `"arguments": false` unless you need them, and never echo the value.
- The authoritative schema of an image's arguments is its `DIVERGE.md` frontmatter:
  `diverge run images read --info --jq '.info? // empty | .schema' '{"image":{…}}'`.

## The `dependencies` member (every agent-loop image)

Every agent-loop image accepts an optional `dependencies` argument: the tool containers the agent
depends on. The program answers its registration with this list unchanged; the daemon deploys each
entry and serves it to the agent as an MCP server. Absent is none.

One entry:

```json
{
  "image": { "digest": "sha256:…", "references": [{ "registry": "docker.io", "name": "diverge-local/memory" }] },
  "extra_memory": 0,
  "extra_disk": 0,
  "arguments": {}
}
```

| member | meaning |
|---|---|
| `image` | the tool's image, same shape as everywhere. It must be a `dependency_tool` image: an agent image or a `standalone_tool` image here fails the registration |
| `extra_memory`, `extra_disk` | required, in bytes: what the tool container gets beyond the `memory` and `disk` its image's `DIVERGE.md` states. The container's limit is the sum; `0` is what the image needs and no more |
| `arguments` | handed to the tool's program verbatim; what it may be is the tool image's `DIVERGE.md` schema. A credential the tool needs is a member of them |
| `fuse_file_mounts`, `fuse_directory_mounts` | optional: the agent's own paths served into the tool (below) |

An entry states no `mode` and no `scope`. The tool image's own `DIVERGE.md` states both, and no entry
says another: read them with `diverge run images read --info`.

`"mode":"lazy"` in the tool image's document — runs only while it is used. With `"scope":"instance"` it is deployed when the
agent's container starts and stopped with the agent's run. With `"scope":"template"`
it is one container for every agent that declares it, started by the first use and stopped once
nothing has used it for the daemon's idle time.

Where the tool image's document says `"mode":"lazy"` and `"scope":"instance"`, and only there, an entry
may mount the agent's own files, live over FUSE; an entry that names a mount of an image of any other
mode or scope is refused:

```json
"fuse_file_mounts":      [{ "agent_path": ["root", "notes.md"], "tool_path": ["data", "notes.md"] }],
"fuse_directory_mounts": [{ "agent_path": ["workspace"],        "tool_path": ["workspace"] }]
```

`"mode":"eager"` in the tool image's document — made once when the first agent that declares it registers, kept
running while any agent is attached, and retired after the daemon's idle time once none is. It mounts
nothing of any agent's. The document's `scope` says how far the tool's container is shared:

| `scope` | one tool per |
|---|---|
| `"instance"` | parent agent |
| `"template"` | dependency template, full stop (every agent that declares it, from any template) |

Rules:

- A dependency has no name. It is identified by the hash of its entry (image digest without
  references, `extra_memory`, `extra_disk`, arguments, mounts). Declaring the same entry twice
  in one list is refused. The same eager tool entry under the same scope key is the same eager tool.
- What a dependency tool may ask of the daemon (its permissions) is **not** in the entry: the tool's
  own program declares it at its registration. Some committed `DIVERGE.md` schemas still show a
  `permissions` member on dependency templates; the current SDK type has none.
- Only agents have dependencies. A tool container's own list is ignored.

## python (`diverge-agents/python`)

An agent that runs Python source instead of calling a model. Each turn runs the source against the
conversation; nothing is sampled. Useful for deterministic agents, echo agents and tests.

`arguments`:

| member | required | meaning |
|---|---|---|
| `python` | yes | the source, verbatim (never trimmed; whitespace is significant). Written to disk at registration and run every turn |
| `requirements` | no | third-party packages: a map of distribution name to `{"operator": "==" \| "!=" \| ">=" \| ">" \| "<=" \| "<" \| "~=", "version": "<PEP 440 version>"}`. Installed once at registration with pip. One constraint per package; compound ranges and environment markers are not representable |
| `dependencies` | no | see above |

The script contract (`diverge-agents/python/HARNESS.md`): the harness supplies the globals
`input` (the conversation; the last item is the newest message, a list of content blocks, or a
`tool_response`), `tools` and `resources`. The script's last bare expression (or else its printed
JSON) must be a list of assistant chunks, for example
`[{"type":"assistant_text_content","text":"…"}]` or
`[{"type":"assistant_tool_call","id":"call_…","name":"<tool>","arguments":"<json string>"}]`. A
script may never emit `user_*`, `usage` or `tool_response` chunks.

Credentials: **none**.

Minimal template body (an echo agent):

```json
{
  "type": "agent",
  "image": { "digest": "<digest>", "references": [{ "registry": "docker.io", "name": "diverge-local/python" }] },
  "extra_memory": 0,
  "extra_disk": 0,
  "arguments": {
    "python": "last = input[-1] if input else []\nsaid = ' '.join(b.get('text', '') for b in last if isinstance(b, dict) and b.get('type') == 'text') if isinstance(last, list) else ''\n[{'type': 'assistant_text_content', 'text': 'echo: ' + said}]\n"
  }
}
```

Tip: build the body with a small script that reads the source from a file and JSON-encodes it, rather
than hand-escaping Python inside JSON inside a shell string.

With a requirement: `"requirements": {"requests": {"operator": ">=", "version": "2.31.0"}}`. A failed
install is a registration error (`kind: "requirements"`), surfaced as the agent's start error.

## lone (`diverge-agents/lone`)

An agent that runs a local [LoNE](https://github.com/AJ-Gonzalez/LoNE) bundle — LFM3 / LFM2.5
converted for Core ML and the Apple Neural Engine — instead of calling an upstream model. Each turn
runs `lone harness` over the conversation.

**macOS only.** The image builds and registers anywhere, but a turn on a Linux provider exits with
`lone targets the Apple Neural Engine and needs macOS`; that is the harness being honest, not a
registration failure. A thin-client mode is tracked as
[LoNE issue #3](https://github.com/AJ-Gonzalez/LoNE/issues/3).

`arguments`:

| member | required | meaning |
|---|---|---|
| `bundle` | yes | the converted bundle's directory inside the container, holding `manifest.json`; checked at registration |
| `system_prompt` | no | the conversation's leading system message |
| `max_new_tokens` | no | generated-token cap per turn, reasoning included; absent is 512 |
| `temperature`, `top_p`, `top_k` | no | sampling; absent is greedy and the whole distribution |
| `thinking` | no | whether the model may think before answering; absent follows the template |
| `dependencies` | no | see above |

Credentials: **none**. The model is on disk.

Minimal template body:

```json
{
  "type": "agent",
  "image": { "digest": "<digest>", "references": [{ "registry": "docker.io", "name": "diverge-local/lone" }] },
  "extra_memory": 8053063680,
  "extra_disk": 0,
  "arguments": { "bundle": "/models/lfm3-1.7b", "temperature": 0.0, "max_new_tokens": 512 }
}
```

The bundle is mounted on the **agent**, not the template:
`volume_mounts: [{"volume_name":"models","volume_relative_path":["lfm3-1.7b"],"volume_mode":"persistent","container_path":["models","lfm3-1.7b"]}]`.
The image states `"memory":536870912`; with this `extra_memory` the container's memory limit is the
sum, 8589934592 bytes (8 GiB). It is a starting point; the 1.7B LUT8 stack measures ~3.5 GB on disk.

## openrouter (`diverge-agents/openrouter`)

An OpenRouter chat-completions loop. Its tools are the caller's MCP servers (attached tools and
dependencies).

`arguments`:

| member | required | meaning |
|---|---|---|
| `model` | yes | the model to route to, in OpenRouter's `vendor/name` form |
| `api_key` | yes | the OpenRouter API key, a string. Held from registration for the container's life and sent as the bearer token of every request. An empty one is refused at registration |
| `system_prompt` | no | sent as the conversation's leading message |
| `temperature`, `top_p`, `top_k`, `top_a`, `min_p` | no | sampling |
| `frequency_penalty`, `presence_penalty`, `repetition_penalty` | no | penalties |
| `max_tokens` | no | cap on generated tokens, excluding reasoning tokens |
| `max_completion_tokens` | no | cap on generated tokens, counting reasoning tokens |
| `stop` | no | a string or a list of strings |
| `logit_bias` | no | map of token id to integer bias |
| `top_logprobs` | no | alternatives to report per token; absent reports no log probabilities |
| `reasoning` | no | `{"enabled": bool, "max_tokens": n, "effort": "none"\|"minimal"\|"low"\|"medium"\|"high"\|"xhigh", "summary_verbosity": "auto"\|"concise"\|"detailed"}`, every member optional |
| `verbosity` | no | `"low"`, `"medium"`, `"high"` or `"max"` |
| `provider` | no | routing preferences: `{"allow_fallbacks": bool, "require_parameters": bool, "order": [...], "only": [...], "ignore": [...], "quantizations": [...]}`, every member optional |
| `context_compression` | no | `"middle_out"` |
| `dependencies` | no | see above |

Credentials: **`api_key`**, required, whatever the other arguments.

Minimal template body:

```json
{
  "type": "agent",
  "image": { "digest": "<digest>", "references": [{ "registry": "docker.io", "name": "diverge-local/openrouter" }] },
  "extra_memory": 0,
  "extra_disk": 0,
  "arguments": { "model": "<vendor/model>", "api_key": "<OpenRouter key>", "system_prompt": "You are a concise assistant." }
}
```

```bash
diverge run agents create '{"template":"<id>","name":"helper"}'
```

## codex (`diverge-agents/codex`)

An OpenAI Codex CLI agent, pre-installed in the image: one `codex exec` per turn on a resumed thread.
The container is the sandbox: Codex always runs with full access and never asks for approval. Its one
MCP server is the caller's tools through the proxy.

`arguments`:

| member | required | meaning |
|---|---|---|
| `model` | yes | the model, in the endpoint's own naming |
| `auth` | yes | how Codex logs in: an object tagged `method`, carrying that method's credential (below). No default, because every method bills someone |
| `effort` | no | `"minimal"`, `"low"`, `"medium"`, `"high"`, `"xhigh"` |
| `reasoning_summary` | no | `"auto"`, `"concise"`, `"detailed"`, `"none"` — what the stream's reasoning chunks carry |
| `verbosity` | no | `"low"`, `"medium"`, `"high"` |
| `web_search` | no | `"disabled"` (the default when absent), `"cached"`, `"indexed"`, `"live"`. Codex searches itself; it is not one of the caller's tools |
| `provider` | no | `{"base_url": "https://…/v1"}`: an endpoint other than OpenAI's, which must speak the Responses API. **Requires `auth.method` = `api_key`**; any other method beside a provider is refused at registration |
| `dependencies` | no | see above |

Credentials: **exactly one, a member of `auth` beside its `method`**. An empty one is refused at
registration, in an error that names the member and never its value.

| `auth` | value |
|---|---|
| `{"method":"api_key","api_key":"…"}` | a string: an OpenAI API key. Handed to Codex as `CODEX_API_KEY` |
| `{"method":"access_token","access_token":"…"}` | a string: a personal access token or an Agent Identity JWT |
| `{"method":"auth_file","auth":{…}}` | a JSON object: Codex's whole `auth.json` (for example a ChatGPT login's tokens) |
| `{"method":"workload_identity","identity_token":"…","federation_rule_id":"…","context":"…"}` | a string: the identity token. `context` is optional |

`auth_file` refreshes: Codex rewrites the login as it runs, and the argument's tokens are spent by the
first refresh. The agent keeps the refreshed login itself, in its own `instance` database, and uses it
for as long as the template's `auth` is the one it was kept for; an agent made from a template with
another `auth` starts from that argument. So one login serves one agent: two agents made from the same
`auth_file` template each refresh the same login and one of them stops logging in. Give each its own.
A ChatGPT login speaks only to OpenAI's own backend, so do not combine `auth_file` with `provider`.

Minimal template body (API key):

```json
{
  "type": "agent",
  "image": { "digest": "<digest>", "references": [{ "registry": "docker.io", "name": "diverge-local/codex" }] },
  "extra_memory": 1610612736,
  "extra_disk": 3221225472,
  "arguments": { "model": "<model>", "auth": { "method": "api_key", "api_key": "<OpenAI key>" } }
}
```

The limits are the sums with what the image states: 2147483648 bytes of memory (2 GiB) and 4294967296
bytes of disk (4 GiB).

With a ChatGPT login instead:
`"arguments":{"model":"<model>","auth":{"method":"auth_file","auth":{…the auth.json content…}}}`.

## cc (`diverge-agents/cc`)

A Claude Code agent: the harness installs the pinned Claude Code at container start (the container
needs network), runs it as a subprocess and serves the loop. Permissions are skipped inside it; the
container is the sandbox.

`arguments`:

| member | required | meaning |
|---|---|---|
| `model` | yes | the model to run |
| `tools` | no | the built-in tools Claude Code is given, one boolean switch each. **Absent or `false` is off**; the whole object absent is every switch off |
| `thinking` | no | whether to think before answering |
| `effort` | no | `"low"`, `"medium"`, `"high"`, `"xhigh"`, `"max"`; meaningful only with `thinking` on |
| `dependencies` | no | see above |

`tools` switches: `bash`, `read`, `write`, `edit`, `notebook_edit`, `glob`, `grep`, `task`,
`send_message`, `list_agents`, `task_output`, `task_stop`, `monitor`, `workflow`, `todo_write`,
`task_create`, `task_get`, `task_update`, `task_list`, `report_findings`, `repl`, `web_fetch`,
`web_search`, `enter_worktree`, `exit_worktree`, `sleep`.

Always on, with no switch: the caller's MCP tools (attached tools and dependencies), the MCP resource
built-ins, `Skill` (skills are directories mounted under `~/.claude/skills`) and `ToolSearch`. Always
off: anything that waits on a user or outlives the run (plan mode, questions, cron, artifacts).

Credentials: **none**. Claude Code logs in by itself and no argument carries a credential. How a login
reaches the container is not established (see the main file's "Not verified").

Minimal template body:

```json
{
  "type": "agent",
  "image": { "digest": "<digest>", "references": [{ "registry": "docker.io", "name": "diverge-local/cc" }] },
  "extra_memory": 3758096384,
  "extra_disk": 7516192768,
  "arguments": {
    "model": "<model>",
    "tools": { "bash": true, "read": true, "write": true, "edit": true, "glob": true, "grep": true }
  }
}
```

The limits are the sums with what the image states: 4294967296 bytes of memory (4 GiB) and 8589934592
bytes of disk (8 GiB).
