---
{
  "type": "agent",
  "name": "codex",
  "description": "An agent that runs the OpenAI Codex CLI.",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Agent",
    "description": "An agent running against Codex.\n\nFew knobs, and not because anything is missing: Codex decides its\nown sampling, its own tools and its own loop, and what it exposes\nto a caller is the model, how hard it reasons and how it speaks,\nwhether it may search the web, where it sends its requests, and\nhow it logs in — the method and, with it, its credential, an\nargument like any other (see [`Auth`]). Each field is\none config key or flag, named in its doc; every `Option` absent\nleaves that key unset, so Codex applies its own default for the\nmodel — except [`web_search`](Self::web_search), where absent is\noff, because nothing is on by omission.",
    "type": "object",
    "properties": {
      "model": {
        "description": "The model to run: `model` (`--model`), in the endpoint's own\nnaming.",
        "type": "string"
      },
      "effort": {
        "description": "How much effort the model spends reasoning:\n`model_reasoning_effort`. See [`Effort`].",
        "anyOf": [
          {
            "$ref": "#/$defs/Effort"
          },
          {
            "type": "null"
          }
        ]
      },
      "reasoning_summary": {
        "description": "How much of its reasoning the model says out loud:\n`model_reasoning_summary`. What the stream's reasoning chunks\ncarry. See [`Summary`].",
        "anyOf": [
          {
            "$ref": "#/$defs/Summary"
          },
          {
            "type": "null"
          }
        ]
      },
      "verbosity": {
        "description": "How long the model's answers run: `model_verbosity`. See\n[`Verbosity`].",
        "anyOf": [
          {
            "$ref": "#/$defs/Verbosity"
          },
          {
            "type": "null"
          }
        ]
      },
      "web_search": {
        "description": "Whether, and how, the model may search the web: `web_search`.\nA capability of Codex itself rather than a tool of the\ncaller's — Codex performs the search and never calls out.\nABSENT IS `disabled`. See [`WebSearch`].",
        "anyOf": [
          {
            "$ref": "#/$defs/WebSearch"
          },
          {
            "type": "null"
          }
        ]
      },
      "provider": {
        "description": "Where requests go instead of OpenAI: a `model_providers` entry\nthe harness writes and selects, keyed by `CODEX_API_KEY` — so\nit requires the `api_key` login, and the registration refuses\nit beside another. Absent is OpenAI's own endpoint. See\n[`Provider`].",
        "anyOf": [
          {
            "$ref": "#/$defs/Provider"
          },
          {
            "type": "null"
          }
        ]
      },
      "auth": {
        "description": "How Codex logs in: one of its own methods, tagged `method`,\neach carrying its credential. REQUIRED — no\nmethod is the default, because every one bills someone. See\n[`Auth`].",
        "$ref": "#/$defs/Auth"
      },
      "dependencies": {
        "description": "The tool containers this agent depends on, each a dependency\ntool template the caller deploys and serves to it as an MCP\nserver, exactly as the SDK's [`Template`] states one: that\ntype's JSON Schema is what `--schema` prints for this member,\nin `/.diverge/DIVERGE.md`, and this list, unchanged, is what the\nregistration answers — one schema, from the one type, at both.\nAbsent is none.",
        "type": "array",
        "items": {
          "$ref": "#/$defs/Template"
        }
      }
    },
    "required": [
      "model",
      "auth"
    ],
    "$defs": {
      "Effort": {
        "description": "How much effort the model spends before answering: Codex's\n`model_reasoning_effort`, its five rungs as Codex spells them.\nThere is no `max`: `xhigh` is the top, and a model that has fewer\nrungs takes the nearest it has.",
        "oneOf": [
          {
            "description": "Least.",
            "type": "string",
            "const": "minimal"
          },
          {
            "description": "Little.",
            "type": "string",
            "const": "low"
          },
          {
            "description": "Balanced.",
            "type": "string",
            "const": "medium"
          },
          {
            "description": "More.",
            "type": "string",
            "const": "high"
          },
          {
            "description": "As much as the model will spend.",
            "type": "string",
            "const": "xhigh"
          }
        ]
      },
      "Summary": {
        "description": "How much of its reasoning the model says out loud: Codex's\n`model_reasoning_summary`. What the run's reasoning chunks carry —\nCodex streams the summary, never the raw reasoning, so this is\nthe whole of what a caller can see of the model thinking.",
        "oneOf": [
          {
            "description": "The model decides.",
            "type": "string",
            "const": "auto"
          },
          {
            "description": "Short.",
            "type": "string",
            "const": "concise"
          },
          {
            "description": "Long.",
            "type": "string",
            "const": "detailed"
          },
          {
            "description": "No summary: the reasoning chunks stay empty.",
            "type": "string",
            "const": "none"
          }
        ]
      },
      "Verbosity": {
        "description": "How long the model's answers run: Codex's `model_verbosity`,\nhonored by the models that take it (the reference names the GPT-5\nfamily on the Responses API) and unset otherwise.",
        "oneOf": [
          {
            "description": "Terse.",
            "type": "string",
            "const": "low"
          },
          {
            "description": "Balanced.",
            "type": "string",
            "const": "medium"
          },
          {
            "description": "Expansive.",
            "type": "string",
            "const": "high"
          }
        ]
      },
      "WebSearch": {
        "description": "Whether, and how, the model may search the web: Codex's\n`web_search`, its four values as Codex spells them. Codex performs\nthe search itself — it is a capability of the upstream, not a tool\nthe caller serves — so a search never appears on the stream as\none of the caller's tool calls. The harness writes `disabled` when\nthe agent says nothing.",
        "oneOf": [
          {
            "description": "No search at all.",
            "type": "string",
            "const": "disabled"
          },
          {
            "description": "Search over a cached index.",
            "type": "string",
            "const": "cached"
          },
          {
            "description": "Search over the live index.",
            "type": "string",
            "const": "indexed"
          },
          {
            "description": "Live fetches of the pages themselves.",
            "type": "string",
            "const": "live"
          }
        ]
      },
      "Provider": {
        "description": "An endpoint other than OpenAI's: one `model_providers` entry the\nharness writes as `model_providers.diverge` and selects with\n`model_provider = \"diverge\"`. The entry's `env_key` is\n`CODEX_API_KEY` — the variable the `api_key` login method hands\nits key to Codex under — and its `wire_api` is\n`responses`, the one value the configuration reference names, so\nthe endpoint must speak the Responses API, which the Diverge relay\nand every upstream this protocol routes do.\n\nAbsent, Codex's own `openai` provider is used, with whatever\nlogin the agent's [`auth`](super::Agent::auth) names. Present, the\nentry REQUIRES the `api_key` method: a ChatGPT login speaks only\nto OpenAI's own backend, and no other method fills an `env_key`,\nso the registration refuses a provider beside any other method\n(`400`, `{\"kind\":\"auth\"}`).",
        "type": "object",
        "properties": {
          "base_url": {
            "description": "The base URL, `/v1` included: `model_providers.diverge.base_url`.",
            "type": "string"
          }
        },
        "required": [
          "base_url"
        ]
      },
      "Auth": {
        "description": "The login method, and with it the ONE credential the agent logs in\nwith.\n\nEach method carries its credential as a member of its own: an\nargument like any other, held from registration for the\ncontainer's life. The four are Codex's own ways in, as its source\nhas them (`rust-v0.163.0`, against the pinned `0.153.4`), and\nnothing the run does with them is invented: a variable Codex\nreads, or a file at the path Codex reads. No credential is empty:\nthe registration refuses one that is, naming the member and never\nits value. [`fmt::Debug`] names the method and nothing else, so no\ncredential is ever formatted.\n\n`OPENAI_API_KEY` in the environment is NOT one of them. Codex's\nbuilt-in `openai` provider has no `env_key` — the variable only\nprefills the interactive login — so a key in the environment under\nthat name logs nothing in. `CODEX_API_KEY` is the variable the\ncredential store reads first, and `codex exec` honours it; the\n`api_key` method is its key handed to Codex under that name.\n\nA ChatGPT login — the `auth_file` method with ChatGPT tokens in\nit — speaks only to OpenAI's own backend, and a\n[`provider`](super::Agent::provider) beside one fails as itself,\nin Codex's words; a provider's entry is keyed by `CODEX_API_KEY`,\nso a provider REQUIRES `api_key`, and the registration refuses\nthe pair otherwise. Tagged `method`, snake case on the wire.",
        "oneOf": [
          {
            "description": "An OpenAI API key, put in the process environment as\n`CODEX_API_KEY`, the top-precedence credential — above\n`auth.json`, which is never written under this method. Bills\nthe platform at API rates. The one method a\n[`provider`](super::Agent::provider) composes with.",
            "type": "object",
            "properties": {
              "api_key": {
                "description": "The key. Not empty.",
                "type": "string"
              },
              "method": {
                "type": "string",
                "const": "api_key"
              }
            },
            "required": [
              "method",
              "api_key"
            ]
          },
          {
            "description": "A personal access token (`at-…`) or an Agent Identity JWT, put\nin the process environment as `CODEX_ACCESS_TOKEN`, where\nCodex's credential store reads it after `CODEX_API_KEY`.",
            "type": "object",
            "properties": {
              "access_token": {
                "description": "The token. Not empty.",
                "type": "string"
              },
              "method": {
                "type": "string",
                "const": "access_token"
              }
            },
            "required": [
              "method",
              "access_token"
            ]
          },
          {
            "description": "Codex's own `auth.json`, whole, written to\n`$CODEX_HOME/auth.json` before every loop and read back after\nthe loop's last turn. Codex rewrites the file in place when a\nChatGPT login refreshes — the three tokens and `last_refresh`\nrotate, five minutes before the access token's expiry, eight\ndays after `last_refresh`, or on a `401` — and what it leaves\nis kept in the agent's own `instance` database, so the next\nloop, and the next start of the container, finds the refreshed\nlogin: see [`login`](crate::login) for the rule. Bills\nwhatever the login is.",
            "type": "object",
            "properties": {
              "auth": {
                "description": "The file's content, a JSON object of any kind the file\ntakes — ChatGPT tokens (`auth_mode`, `tokens {id_token,\naccess_token, refresh_token, account_id}`, `last_refresh`),\nan `OPENAI_API_KEY` member, a `personal_access_token`, an\n`agent_identity`, Bedrock credentials. Not empty.",
                "type": "object",
                "additionalProperties": true
              },
              "method": {
                "type": "string",
                "const": "auth_file"
              }
            },
            "required": [
              "method",
              "auth"
            ]
          },
          {
            "description": "Workload identity federation: an identity token exchanged\nagainst a federation rule. The token is written to\n`$CODEX_HOME/identity-token` and named to Codex by\n`OPENAI_IDENTITY_TOKEN_FILE`, with `OPENAI_FEDERATION_RULE_ID`\nand, when given, `OPENAI_WORKLOAD_IDENTITY_CONTEXT` from the\ntwo members beside it.",
            "type": "object",
            "properties": {
              "method": {
                "type": "string",
                "const": "workload_identity"
              },
              "identity_token": {
                "description": "The identity token. Not empty.",
                "type": "string"
              },
              "federation_rule_id": {
                "description": "The federation rule the token is exchanged against:\n`OPENAI_FEDERATION_RULE_ID`.",
                "type": "string"
              },
              "context": {
                "description": "The workload's context, when the rule wants one:\n`OPENAI_WORKLOAD_IDENTITY_CONTEXT`. Absent leaves the\nvariable unset.",
                "type": [
                  "string",
                  "null"
                ]
              }
            },
            "required": [
              "method",
              "identity_token",
              "federation_rule_id"
            ]
          }
        ]
      },
      "Template": {
        "description": "One tool container an agent's program depends on: everything the\ncaller needs to deploy it that the tool's image does not say for\nitself, as fields and not as words. A tool container depends on\nnothing.\n\nA [`Container`](crate::shared::containers::request::Container) is an\nimage, two limits, mounts and arguments. The image and the\narguments are here, member for member. The limits are the image's\nown — the `memory` and the `disk` of its\n[document](crate::shared::containers::info::Document) — and what\nthis adds to each. The mounts are not the caller's volumes — a\nvolume is named in the caller's own listing, and a tool that named\none would be dictating the caller's storage — but the AGENT's own\nfiles and directories, each a path in the agent container served\nlive at a path in the tool container: the caller serves the agent\ncontainer, a `containers::fuse_serve` of the path, and mounts the\ntool, a `fuse::mount` answered from it.\n\n# What the image says, and this does not\n\nThe image is a dependency tool's: its document's `type` is\n`dependency_tool`, and a declaration of an image of another kind —\nan agent's, or a standalone tool's — is the registration's error.\nThat document, and not this, says:\n\n1. How the tool is kept running, its\n   [`mode`](crate::shared::containers::info::Mode): lazy or eager.\n2. How far it is shared, its\n   [`scope`](crate::shared::containers::info::Scope): one per\n   agent, one per agent template, or one for all.\n3. The memory and the disk it needs.\n\nSo two agents that declare one image declare one kind of tool, and\nno declaration says of an image what the image does not.\n\n# The mounts are a lazy tool's of one agent\n\nA mount is the declaring agent's own path, so there must be one\nagent whose path it is, and that agent's run must be the tool's:\nonly an image whose `mode` is `lazy` and whose `scope` is\n`instance` is declared with any. A declaration that names a mount\nof an image of any other mode or scope is the registration's\nerror.\n\nWhat the tool may ask of the caller is not here: the tool's own\nprogram declares it at its registration, every time it starts, as a\nstandalone tool's does — see the `permissions` of the\n[`register`](crate::container_proxy::inside::register) answer.\n\nThere are no instructions. What a declaration cannot say — the\nprovider the tool runs on, the volumes it is given — is the\ncaller's, and the caller needs no prose to choose it. The daemon's\nown tool templates are another thing: a record the daemon keeps,\nnamed by a hash, made by a request, of a standalone tool's image;\nthis is what an agent's program declares and a caller deploys.\n\nThere is no name. A dependency is named by its ID: the lowercase\nhexadecimal SHA-256 of its [`canonical`](crate::shared::canonical)\nbytes — this template as compact JSON, absent members omitted, the\nimage's `references` absent too, every object key sorted at every\ndepth, the `arguments` and the mounts included — sixty-four characters, which the caller computes over\n[`hashed`](Self::hashed). Two declarations with one id are one\ndependency, and a list that declares one twice is refused; the same\ntemplate deployed for the same agent, on whatever provider, is the\nsame dependency, and the same template declared under the same\nscope key, by whatever agent, is the same tool. The caller labels\nthe tool's MCP tools by the id when it merges lists, and the\nprogram knows which tool serves each by the keys under `_meta`, see\n[`shared::mcp`](crate::shared::mcp).",
        "type": "object",
        "properties": {
          "image": {
            "description": "The image: a digest and its references, as a\n`containers::tools::run` request names one. A dependency tool's.",
            "$ref": "#/$defs/Image"
          },
          "extra_memory": {
            "description": "The memory the tool container has beyond what its image says it\nneeds, in bytes. `0` gives it what the image needs and no more.\nThe container's limit is the image's own `memory` and this,\nsummed: a sum past 2 to the power 64 less 1 is the\nregistration's error.",
            "type": "integer",
            "format": "uint64",
            "minimum": 0
          },
          "extra_disk": {
            "description": "The bytes the tool container may write beyond what its image\nsays it needs. `0` gives it what the image needs and no more.\nThe container's limit is the image's own `disk` and this,\nsummed, as [`extra_memory`](Self::extra_memory) is.",
            "type": "integer",
            "format": "uint64",
            "minimum": 0
          },
          "arguments": {
            "description": "The arguments the tool container is registered with — the\n`arguments` of the run request that makes it, verbatim."
          },
          "fuse_file_mounts": {
            "description": "Files of the agent container served live into the tool\ncontainer, one each over FUSE: see [`Mount`]. The agent's path\nnames a file; the tool's path is where it appears. Only of an\nimage whose `mode` is `lazy` and whose `scope` is `instance`.\nAbsent when empty.",
            "type": "array",
            "items": {
              "$ref": "#/$defs/Mount"
            }
          },
          "fuse_directory_mounts": {
            "description": "Directories of the agent container served live into the tool\ncontainer, one each over FUSE: see [`Mount`]. The agent's path\nnames a directory, whose whole tree the tool sees; the tool's\npath is where it appears. Only of an image whose `mode` is\n`lazy` and whose `scope` is `instance`. Absent when empty.",
            "type": "array",
            "items": {
              "$ref": "#/$defs/Mount"
            }
          }
        },
        "required": [
          "image",
          "extra_memory",
          "extra_disk",
          "arguments"
        ]
      },
      "Image": {
        "description": "The image, by its manifest digest and the references under which\nits bytes may be fetched — what\n[`images::read`](crate::provider::endpoints::images::read::client::request::Frame)\nasks about, so a read scope that came back held names an image a\nrun can ask for, with nothing to translate between them. The same\nvalue is the whole of the shared\n[`oci::has`](crate::shared::containers::oci::has) request a\nprovider asks a caller on a run, and the image every\n[`oci::prove`](crate::shared::containers::oci::prove) ask on a\nscope naming it is over.\n\n# The digest is the image\n\nA digest cannot be repointed at different content, so an image\nmeans the same bytes every time, wherever they come from. That is\nthe whole of what the wire asserts: the provider runs the image\nthe digest names and no other. The references are not the image:\nthe same bytes sit under different names on different registries,\nand an id that hashes an image hashes it with none — see\n[`hashed`](Self::hashed).\n\n# Where the bytes come from\n\nA provider looks in the caller's own Store first, by digest alone —\nthe images the caller\n[built](crate::provider::endpoints::images::build) or\n[pushed](crate::provider::endpoints::images::push) on it, and no\nother caller's. Failing that it asks any referenced registry its\nown policy lists, with the credential it holds for it, and ignores\na reference naming one it does not; and failing that it takes the\nimage from the caller — asking on the run scope whether the caller\nholds it, naming this image, and, when it does, for its manifest\nand blobs, then for the proof that it holds it, see\n[`oci`](crate::shared::containers::oci). The order, and which\nanswered, is stated on\n[`Source`](crate::shared::containers::response::Source), which the\nrun's answer carries. A caller need not hold what it names; one\nthat does answers so when asked.",
        "type": "object",
        "properties": {
          "digest": {
            "description": "The manifest digest, `<algorithm>:<hex>`.\n\nWhat identifies the image: a runtime hashes what it pulls, so\na source serving other bytes under it fails before anything\nruns.",
            "type": "string"
          },
          "references": {
            "description": "Where the bytes may be fetched, each a registry and a path\nthere: see [`Reference`]. Absent when there are none, and then\nthe provider has only what it holds and what the caller holds.",
            "type": "array",
            "items": {
              "$ref": "#/$defs/Reference"
            }
          }
        },
        "required": [
          "digest"
        ]
      },
      "Reference": {
        "description": "One place the bytes an image's digest names may be fetched from: a\nregistry, and the repository path there. A reference locates and\ndoes not identify — the digest does — so an image may carry none,\none or several, and two images with one digest are one image\nwhatever each lists. A provider asks a referenced registry only\nwhere its own policy lists that registry, with the credential it\nholds for it, and ignores a reference naming one it does not.",
        "type": "object",
        "properties": {
          "registry": {
            "description": "The registry's host, as a reference names it before its first\n`/`: `docker.io`, `ghcr.io`, `registry.example.com:5000`. No\nscheme and no path.",
            "type": "string"
          },
          "name": {
            "description": "The repository path at that registry — `library/nginx`,\n`myorg/myimage`. No host and no tag: the image is\n`<registry>/<name>@<digest>` there.\n\nIt lands in a reference by concatenation wherever a provider\npulls from, so a provider refuses one that is not a repository\npath and normalizes nothing.",
            "type": "string"
          }
        },
        "required": [
          "registry",
          "name"
        ]
      },
      "Mount": {
        "description": "A path of the agent container, mounted at a path of the tool\ncontainer over FUSE: source before destination, the order a mount\nreads in everywhere else. Which of the two it is — a file or a\ndirectory — is which list of the [`Template`](super::Template) it\nis on.\n\nThe caller serves the agent container's path live — a\n`containers::fuse_serve` of it, or of the directory holding a file —\nand makes the mount in the tool container as a FUSE mount answered\nfrom that FUSE serve, so every read, write, listing, removal, rename and\nnew directory the tool makes is one ask answered from the agent's\nown filesystem, where the agent sees it at once. Nothing is copied:\nthe two containers share the path for as long as both run.",
        "type": "object",
        "properties": {
          "agent_path": {
            "description": "Where it is in the agent container, as path components from the\nagent container's root — the shape every path in this crate\ntakes. No component is empty, `.` or `..`, and the path is not\nthe root: a file on the file list, a directory on the directory\nlist.",
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "tool_path": {
            "description": "Where it appears in the tool container, as path components from\nthe tool container's root, under the rules a\n[`Container`](crate::shared::containers::request::Container)\nstates for its mounts: not the root, no other mount's, inside\nno other mount's, and no other mount's inside it.",
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        },
        "required": [
          "agent_path",
          "tool_path"
        ]
      }
    }
  },
  "memory": 536870912,
  "disk": 1073741824,
  "chat": {
    "prompt_modalities": {
      "image": true,
      "audio": false,
      "file": false
    }
  },
  "structured_decisions": false
}
---
# Codex

An agent container running an OpenAI Codex CLI agent, pre-installed
in the image, one `codex exec` per turn on a resumed thread.

Credentials: one, an argument beside `auth.method` — `api_key` takes
`api_key`; `access_token` takes `access_token`; `auth_file` takes
`auth`, the whole `auth.json`, and keeps what Codex refreshes it to in
the agent's own database; `workload_identity` takes `identity_token`.
