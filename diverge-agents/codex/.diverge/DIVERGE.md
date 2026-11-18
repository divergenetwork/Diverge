---
{
  "type": "agent",
  "name": "codex",
  "chat": {
    "prompt_modalities": {
      "image": true,
      "audio": false,
      "file": false
    }
  },
  "structured_decisions": false,
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Agent",
    "description": "An agent running against Codex.\n\nFew knobs, and not because anything is missing: Codex decides its\nown sampling, its own tools and its own loop, and what it exposes\nto a caller is the model, how hard it reasons and how it speaks,\nwhether it may search the web, where it sends its requests, and\nhow it logs in â€” the method only, never the credential, which is\nthe one secret the method declares (see [`Auth`]). Each field is\none config key or flag, named in its doc; every `Option` absent\nleaves that key unset, so Codex applies its own default for the\nmodel â€” except [`web_search`](Self::web_search), where absent is\noff, because nothing is on by omission.",
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
        "description": "Whether, and how, the model may search the web: `web_search`.\nA capability of Codex itself rather than a tool of the\ncaller's â€” Codex performs the search and never calls out.\nABSENT IS `disabled`. See [`WebSearch`].",
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
        "description": "Where requests go instead of OpenAI: a `model_providers` entry\nthe harness writes and selects, keyed by `CODEX_API_KEY` â€” so\nit requires the `api_key` login, and the registration refuses\nit beside another. Absent is OpenAI's own endpoint. See\n[`Provider`].",
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
        "description": "How Codex logs in: one of its own methods, tagged `method`,\neach declaring the one secret the proxy feeds. REQUIRED â€” no\nmethod is the default, because every one bills someone. See\n[`Auth`].",
        "$ref": "#/$defs/Auth"
      },
      "dependencies": {
        "description": "The tool containers this agent depends on, each a dependency\ntool template the caller deploys and serves to it as an MCP\nserver, exactly as the SDK's [`Template`] states one: that\ntype's JSON Schema is what `--schema` prints for this member,\nin `/.diverge/DIVERGE.md`, and this list, unchanged, is what the\nregistration answers â€” one schema, from the one type, at both.\nAbsent is none.",
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
        "description": "How much of its reasoning the model says out loud: Codex's\n`model_reasoning_summary`. What the run's reasoning chunks carry â€”\nCodex streams the summary, never the raw reasoning, so this is\nthe whole of what a caller can see of the model thinking.",
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
        "description": "Whether, and how, the model may search the web: Codex's\n`web_search`, its four values as Codex spells them. Codex performs\nthe search itself â€” it is a capability of the upstream, not a tool\nthe caller serves â€” so a search never appears on the stream as\none of the caller's tool calls. The harness writes `disabled` when\nthe agent says nothing.",
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
        "description": "An endpoint other than OpenAI's: one `model_providers` entry the\nharness writes as `model_providers.diverge` and selects with\n`model_provider = \"diverge\"`. The entry's `env_key` is\n`CODEX_API_KEY` â€” the variable the `api_key` login method hands\nthe fed `OPENAI_API_KEY` to Codex under â€” and its `wire_api` is\n`responses`, the one value the configuration reference names, so\nthe endpoint must speak the Responses API, which the Diverge relay\nand every upstream this protocol routes do.\n\nAbsent, Codex's own `openai` provider is used, with whatever\nlogin the agent's [`auth`](super::Agent::auth) names. Present, the\nentry REQUIRES the `api_key` method: a ChatGPT login speaks only\nto OpenAI's own backend, and no other method fills an `env_key`,\nso the registration refuses a provider beside any other method\n(`400`, `{\"kind\":\"auth\"}`).",
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
        "description": "The login method, and through it the ONE secret the agent takes.\n\nThe agent names the method and never a credential: each method\ndeclares one secret, by a fixed name the template must declare,\nand the value the caller binds to that name is what Codex logs in\nwith. The four are Codex's own ways in, as its source has them\n(`rust-v0.163.0`, against the pinned `0.153.4`), and nothing the\nrun does with them is invented: a variable Codex reads, or a file\nat the path Codex reads.\n\n`OPENAI_API_KEY` in the environment is NOT one of them. Codex's\nbuilt-in `openai` provider has no `env_key` â€” the variable only\nprefills the interactive login â€” so a key in the environment under\nthat name logs nothing in. `CODEX_API_KEY` is the variable the\ncredential store reads first, and `codex exec` honours it; the\n`api_key` method is the fed `OPENAI_API_KEY` handed to Codex under\nthat name.\n\nA ChatGPT login â€” the `auth_file` method with ChatGPT tokens in\nit â€” speaks only to OpenAI's own backend, and a\n[`provider`](super::Agent::provider) beside one fails as itself,\nin Codex's words; a provider's entry is keyed by `CODEX_API_KEY`,\nso a provider REQUIRES `api_key`, and the registration refuses\nthe pair otherwise. Tagged `method`, snake case on the wire.",
        "oneOf": [
          {
            "description": "An OpenAI API key. Declares `OPENAI_API_KEY`, `fixed`: a JSON\nstring, put in the process environment as `CODEX_API_KEY`,\nthe top-precedence credential â€” above `auth.json`, which is\nnever written under this method. Bills the platform at API\nrates. The one method a [`provider`](super::Agent::provider)\ncomposes with.",
            "type": "object",
            "properties": {
              "method": {
                "type": "string",
                "const": "api_key"
              }
            },
            "required": [
              "method"
            ]
          },
          {
            "description": "A personal access token (`at-â€¦`) or an Agent Identity JWT.\nDeclares `CODEX_ACCESS_TOKEN`, `fixed`: a JSON string, put in\nthe process environment under the same name, where Codex's\ncredential store reads it after `CODEX_API_KEY`.",
            "type": "object",
            "properties": {
              "method": {
                "type": "string",
                "const": "access_token"
              }
            },
            "required": [
              "method"
            ]
          },
          {
            "description": "Codex's own `auth.json`, whole. Declares `CODEX_AUTH`,\n`rotating`: a JSON object of any kind the file takes â€” ChatGPT\ntokens (`auth_mode`, `tokens {id_token, access_token,\nrefresh_token, account_id}`, `last_refresh`), an `OPENAI_API_KEY`\nmember, a `personal_access_token`, an `agent_identity`, Bedrock\ncredentials â€” written to `$CODEX_HOME/auth.json` before every\nloop and read back after the loop's last turn. Codex rewrites\nthe file in place when a ChatGPT login refreshes â€” the three\ntokens and `last_refresh` rotate, five minutes before the\naccess token's expiry, eight days after `last_refresh`, or on a\n`401` â€” and what it leaves is what the collect answers, so the\nnext activation finds the refreshed login. Bills whatever the\nlogin is.",
            "type": "object",
            "properties": {
              "method": {
                "type": "string",
                "const": "auth_file"
              }
            },
            "required": [
              "method"
            ]
          },
          {
            "description": "Workload identity federation: an identity token exchanged\nagainst a federation rule. Declares `OPENAI_IDENTITY_TOKEN`,\n`fixed`: a JSON string, written to `$CODEX_HOME/identity-token`\nand named to Codex by `OPENAI_IDENTITY_TOKEN_FILE`, with\n`OPENAI_FEDERATION_RULE_ID` and, when given,\n`OPENAI_WORKLOAD_IDENTITY_CONTEXT` from the two members here.",
            "type": "object",
            "properties": {
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
              },
              "method": {
                "type": "string",
                "const": "workload_identity"
              }
            },
            "required": [
              "method",
              "federation_rule_id"
            ]
          }
        ]
      },
      "Template": {
        "description": "One tool container an agent's program depends on: everything the\ncaller needs to deploy it, as fields and not as words. A tool\ncontainer depends on nothing.\n\nA [`Container`](crate::shared::containers::request::Container) is an\nimage, two limits, mounts and arguments. The image, the limits and\nthe arguments are here, member for member; beside them the secrets\nthe image needs, by name alone — which of them the tool's program\ntakes, and in what mode, is its own answer at registration, and\ntheir values are the caller's, each resolved through the vault key\nthe agent that spawns the tool binds the same name to, fed at every\nactivation. The mounts are not the caller's volumes — a volume is\nnamed in the caller's own listing, and a tool that named one would\nbe dictating the caller's storage — but, for a lazy tool of the\n`instance` scope alone,\nthe AGENT's own files and directories, each a path in the agent\ncontainer served live at a path in the tool container: the caller\nserves the agent container, a `containers::fuse_serve` of the path,\nand mounts the tool, a `fuse::mount` answered from it.\n[`mode`](Self::mode) says which of the two modes the dependency is —\na lazy tool, run while it is used, or an eager one the caller\nkeeps, each with a scope — see [`Mode`]. What the tool may ask\nof the caller is not here: the tool's own program declares it at its\nregistration, every time it starts, as a standalone tool's does —\nsee the `permissions` of the\n[`register`](crate::container_proxy::inside::register) answer.\n\nThere are no instructions. What a template cannot say — the provider\nthe tool runs on, the volumes it is given — is\nthe caller's, and the caller needs no prose to choose it. The\ndaemon's own tool templates are another thing: a record the daemon\nkeeps, named by a hash, made by a request; this is what an image\ndeclares and a caller deploys.\n\nThere is no name. A dependency is named by its ID: the lowercase\nhexadecimal SHA-256 of its [`canonical`](crate::shared::canonical)\nbytes — this template as compact JSON, absent members omitted, the\nimage's `references` absent too, every object key sorted at every\ndepth, the `arguments` and the `secrets` included, the names sorted\nbytewise, the `mode` and\nits members among the keys — sixty-four characters, which the caller\ncomputes over [`hashed`](Self::hashed). Two declarations with one id\nare one dependency, and a list that declares one twice is refused;\nthe same template deployed for the same agent, on whatever provider,\nis the same dependency, and the same template declared under\nthe same scope key, by whatever agent, is the same tool. The\ncaller labels the tool's MCP tools by the id when it merges lists,\nand the program knows which tool serves each by the keys under\n`_meta`, see [`shared::mcp`](crate::shared::mcp).",
        "type": "object",
        "properties": {
          "image": {
            "description": "The image: a digest and its references, as a\n`containers::tools::run` request names one.",
            "$ref": "#/$defs/Image"
          },
          "memory": {
            "description": "The memory the tool container needs, in bytes.",
            "type": "integer",
            "format": "uint64",
            "minimum": 0
          },
          "disk": {
            "description": "The bytes the tool container may write, in bytes.",
            "type": "integer",
            "format": "uint64",
            "minimum": 0
          },
          "arguments": {
            "description": "The arguments the tool container is registered with — the\n`arguments` of the run request that makes it, verbatim."
          },
          "secrets": {
            "description": "The secrets the image needs, by name, in the order the image\nstates them: names only. Which of them the tool's program takes\nand in what mode is the program's answer at its registration —\nsee [`secrets`](crate::shared::containers::secrets) — and the\nvalue of each is resolved through the vault key the agent the\ntool is spawned for binds the same name to, fed at every\nactivation. Inside the hash, sorted bytewise there. A name is\nnot empty, no two are alike, and every one is a name the\nregistering agent's template declares; a template that says\notherwise is the registration's error. Absent, the image needs\nnone.",
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        },
        "required": [
          "image",
          "memory",
          "disk",
          "arguments"
        ],
        "oneOf": [
          {
            "description": "A tool run while it is used: `\"mode\":\"lazy\"`.",
            "type": "object",
            "properties": {
              "scope": {
                "description": "How far the tool is shared, and its database with it: see\n[`Scope`]. Required.",
                "$ref": "#/$defs/Scope"
              },
              "fuse_file_mounts": {
                "description": "Files of the agent container served live into the tool\ncontainer, one each over FUSE: see [`Mount`]. The agent's\npath names a file; the tool's path is where it appears.\nOnly where `scope` is `instance`. Absent when empty.",
                "type": "array",
                "items": {
                  "$ref": "#/$defs/Mount"
                }
              },
              "fuse_directory_mounts": {
                "description": "Directories of the agent container served live into the\ntool container, one each over FUSE: see [`Mount`]. The\nagent's path names a directory, whose whole tree the tool\nsees; the tool's path is where it appears. Only where\n`scope` is `instance`. Absent when empty.",
                "type": "array",
                "items": {
                  "$ref": "#/$defs/Mount"
                }
              },
              "mode": {
                "type": "string",
                "const": "lazy"
              }
            },
            "required": [
              "mode",
              "scope"
            ]
          },
          {
            "description": "A tool the caller keeps: `\"mode\":\"eager\"`.",
            "type": "object",
            "properties": {
              "scope": {
                "description": "How far the tool is shared, and its database with it: see\n[`Scope`]. Required.",
                "$ref": "#/$defs/Scope"
              },
              "mode": {
                "type": "string",
                "const": "eager"
              }
            },
            "required": [
              "mode",
              "scope"
            ]
          }
        ]
      },
      "Image": {
        "description": "The image, by its manifest digest and the references under which\nits bytes may be fetched — what\n[`images::oci`](crate::provider::endpoints::images::oci::client::request::Frame)\nasks about, so an oci scope that came back held names an image a\nrun can ask for, with nothing to translate between them. The same\nvalue is the whole of the shared\n[`oci::has`](crate::shared::containers::oci::has) request a\nprovider asks a caller on a run, and the image every\n[`oci::prove`](crate::shared::containers::oci::prove) ask on a\nscope naming it is over.\n\n# The digest is the image\n\nA digest cannot be repointed at different content, so an image\nmeans the same bytes every time, wherever they come from. That is\nthe whole of what the wire asserts: the provider runs the image\nthe digest names and no other. The references are not the image:\nthe same bytes sit under different names on different registries,\nand an id that hashes an image hashes it with none — see\n[`hashed`](Self::hashed).\n\n# Where the bytes come from\n\nA provider looks in the caller's own Store first, by digest alone —\nthe images the caller\n[built](crate::provider::endpoints::images::build) or\n[pushed](crate::provider::endpoints::images::push) on it, and no\nother caller's. Failing that it asks any referenced registry its\nown policy lists, with the credential it holds for it, and ignores\na reference naming one it does not; and failing that it takes the\nimage from the caller — asking on the run scope whether the caller\nholds it, naming this image, and, when it does, for its manifest\nand blobs, then for the proof that it holds it, see\n[`oci`](crate::shared::containers::oci). The order, and which\nanswered, is stated on\n[`Source`](crate::shared::containers::response::Source), which the\nrun's answer carries. A caller need not hold what it names; one\nthat does answers so when asked.",
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
      },
      "Scope": {
        "description": "What a dependency tool is one of: one per parent agent, one per\nparent agent template, or one for every agent that declares it,\nwhatever template they are made from. The scope names the tool —\nthe same template declared under the same scope key is the same\ntool, joined and not made again — and names its database scope\nwith it, so what a tool holds is shared exactly as far as the tool\nis. Both modes say one: see\n[`Mode`](super::Mode). Snake case on the wire: `\"instance\"`,\n`\"template\"`, `\"global\"`.",
        "oneOf": [
          {
            "description": "One tool per parent agent: the tool deployed for one agent is\nits own, with a database of its own, and two agents' tools of\nthis template never see each other's. The one scope under\nwhich a lazy tool mounts the agent's files.",
            "type": "string",
            "const": "instance"
          },
          {
            "description": "One tool per parent agent template: every agent made from the\nsame agent template that declares this template is attached\nto one tool, which holds one database, so what one writes the\nothers read.",
            "type": "string",
            "const": "template"
          },
          {
            "description": "One tool for the template, full stop: every agent that\ndeclares it, from whatever agent template, is attached to the\none tool, and its database is one for all of them.",
            "type": "string",
            "const": "global"
          }
        ]
      }
    }
  }
}
---
# Codex

An agent container running an OpenAI Codex CLI agent, pre-installed
in the image, one `codex exec` per turn on a resumed thread.

Secrets: one, chosen by `auth.method` — `api_key` declares
`OPENAI_API_KEY` (fixed); `access_token` declares `CODEX_ACCESS_TOKEN`
(fixed); `auth_file` declares `CODEX_AUTH` (rotating, the whole
`auth.json`); `workload_identity` declares `OPENAI_IDENTITY_TOKEN`
(fixed).
