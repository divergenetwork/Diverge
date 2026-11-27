# hermes (`diverge-agents/hermes`)

A Hermes agent (Nous Research's agent harness): `hermes gateway` laid down from the arguments and
driven over its API. Hermes is multi-provider; the `provider` argument picks the inference source and
**carries the credential** its auth needs. A toolset that needs a credential carries its own.

## `arguments`

| member | required | meaning |
|---|---|---|
| `provider` | yes | the inference source: an object whose `provider` member is the provider id, plus that provider's endpoint facts and credential (table below) |
| `model` | yes | the model, in the provider's own naming |
| `system_prompt` | no | applied per request |
| `effort` | no | `"none"`, `"minimal"`, `"low"`, `"medium"`, `"high"`, `"xhigh"`, `"max"`, `"ultra"`. Absent is the provider's default; `"none"` is reasoning explicitly disabled |
| `toolsets` | no | Hermes's local capabilities, one member each (below). Absent is everything off |
| `dependencies` | no | dependency tool templates; see [agent-loop-images.md](agent-loop-images.md) |

There is no temperature and no thinking-token budget. Skills are not an argument: they arrive as
mounts under `/root/.hermes/external-skills/`, one directory per skill holding its `SKILL.md`.

## Credentials

Every credential is a member of the provider's object or of a toolset's object, required wherever
that provider or toolset is chosen (two are optional, noted below). There is no other place for one.

- Most are strings. An `oauth` member is a JSON object: the login document.
- A login that refreshes (`oauth`) is kept by the agent itself, in its own `instance` database, as
  the last run left it. The kept login is used while the template's `oauth` is the one it was kept
  for; an agent made from a template with another `oauth` starts from that argument. So one login
  serves one agent: give each agent its own.
- Two members that set the same underlying key must carry the same value: `fal_key` of `image_gen`
  and of `video_gen`, and `api_key` of the `xai` provider and of `x_search`. Different values are
  refused at registration.
- Whoever may see the template's arguments sees every credential in it. An error names the member,
  never its value.

## Providers

`provider` is `{"provider":"<id>", …}`.

Each of these takes exactly `api_key` (a string, required) beside the id:

`actual`, `ai-gateway`, `alibaba`, `alibaba-coding-plan`, `anthropic`, `arcee`, `commandcode`,
`commandcode-anthropic`, `deepinfra`, `deepseek`, `fireworks`, `gemini`, `gmi`, `huggingface`,
`kilocode`, `kimi-coding`, `kimi-coding-cn`, `meta-ai`, `minimax`, `minimax-cn`,
`nebius-token-factory`, `novita`, `nvidia`, `ollama-cloud`, `opencode-go`, `opencode-zen`,
`openrouter` (Hermes discards keys that do not start with `sk-or-`), `router`, `stepfun`, `upstage`,
`xai`, `xiaomi`.

The others:

| `provider` id | members beside the id | credential |
|---|---|---|
| `azure-foundry` | `base_url` (required), `api_key` (required) | `api_key` |
| `bedrock` | `auth` (required, below), `region` (optional) | inside `auth` |
| `copilot` | `github_token` (required) | `github_token`: an OAuth or fine-grained token (`gho_`, `ghu_`, `github_pat_`); classic `ghp_` tokens are rejected by Hermes |
| `custom` | `base_url` (required): any OpenAI-compatible endpoint; `api_key` (optional string) | `api_key`, only where the endpoint needs one |
| `minimax-oauth` | `oauth` (required, object) | `oauth` (refreshes) |
| `nous` | `oauth` (required, object) | `oauth` (refreshes) |
| `openai-codex` | `oauth` (required, object) | `oauth` (refreshes) |
| `opencode-free` | none | none (keyless) |
| `qwen-oauth` | `oauth` (required, object: the CLI's token file) | `oauth` (refreshes) |
| `vertex` | `service_account_json` (required), `project_id`, `region` (both optional) | `service_account_json`: the service-account key document **as a JSON string holding the document's text**, not as an object |
| `zai` | `api_key` (required), `base_url` (optional) | `api_key` |

Bedrock's `auth` is an object tagged by `type`:

| `auth` | members |
|---|---|
| `{"type":"bearer-token", …}` | `bearer_token` |
| `{"type":"access-key", …}` | `access_key_id`, `secret_access_key` |
| `{"type":"session-token", …}` | `access_key_id`, `secret_access_key`, `session_token` |

The provider ids are the kebab-case spellings above; confirm the exact member names for a provider
with the image's schema (`diverge run images read --info --jq '.info? // empty | .schema'`).

## Toolsets

`toolsets` is an object; every member absent is off. A plain switch takes `true`. A structured
toolset is on when its object is present.

| member | value | credential |
|---|---|---|
| `terminal` | `true` | none |
| `file` | `true` | none |
| `code_execution` | `true` | none |
| `vision` | `true` | none (rides the run's inference provider) |
| `video` | `true` | none |
| `todo` | `true` | none |
| `session_search` | `true` | none |
| `memory` | `true` | none (Hermes's own `MEMORY.md` / `USER.md`) |
| `image_gen` | `{"fal_key":"…"}` | `fal_key` |
| `video_gen` | `{"fal_key":"…"}` | `fal_key` (the same value as `image_gen`'s when both are on) |
| `x_search` | `{"api_key":"…"}` | `api_key`: an xAI key (the same value as the `xai` provider's when both are used) |
| `web` | `{}` for the keyless ring; `{"search":{"backend":"tavily" \| "exa" \| "parallel" \| "keenable" \| "brave-free","api_key":"…"}}`; `{"search":{"searxng_url":"…"}}`; `{"extract":{"backend":"firecrawl","api_key":"…"}}`; or both slots | the `api_key` of each keyed slot; none for SearXNG |
| `browser` | `{}` for the container's own headless Chromium; `{"remote":{"api_key":"…","project_id":"…"}}` for Browserbase; `{"remote":{"cdp_url":"…"}}` | Browserbase's `api_key` and `project_id`; none otherwise |
| `tts` | `{}` for the free voice; `{"elevenlabs_api_key":"…"}` for ElevenLabs | `elevenlabs_api_key` (optional) |
| `homeassistant` | `{"url":"…","token":"…"}` (both required) | `token` |
| `spotify` | `{"client_id":"…","oauth":{…}}` (both required) | `oauth`: the login document (refreshes) |

The caller's MCP tools (attached tools and dependencies) are always wired in and are not switches.
Sub-agent delegation, cron, clarify and the Discord toolsets are off by design and cannot be turned
on.

## Minimal template body

```json
{
  "type": "agent",
  "image": { "digest": "<digest>", "references": [{ "registry": "docker.io", "name": "diverge-local/hermes" }] },
  "extra_memory": 3758096384,
  "extra_disk": 7516192768,
  "arguments": {
    "provider": { "provider": "openrouter", "api_key": "<OpenRouter key>" },
    "model": "<vendor/model>",
    "toolsets": { "terminal": true, "file": true }
  }
}
```

The image states `"memory":536870912,"disk":1073741824`. The container's limits are the sums:
4294967296 bytes of memory (4 GiB) and 8589934592 bytes of disk (8 GiB).

```bash
diverge run agents create '{"template":"<id>","name":"hermes-1"}'
```

A fuller one, with a keyed search backend and a local OpenAI-compatible endpoint that needs no key:

```json
"arguments": {
  "provider": { "provider": "custom", "base_url": "http://10.0.0.5:11434/v1" },
  "model": "<model>",
  "effort": "medium",
  "toolsets": { "web": { "search": { "backend": "tavily", "api_key": "<Tavily key>" } }, "terminal": true, "file": true, "memory": true }
}
```
