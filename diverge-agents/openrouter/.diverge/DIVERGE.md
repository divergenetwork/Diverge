---
{
  "type": "agent",
  "name": "openrouter",
  "chat": {
    "prompt_modalities": {
      "image": true,
      "audio": true,
      "file": true
    }
  },
  "structured_decisions": false,
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Agent",
    "description": "An agent running against OpenRouter.\n\nThe widest parameter set of any upstream, because OpenRouter fronts\nmany providers and exposes what each of them accepts.\n\n# Secrets\n\nOne, whatever the arguments: [`API_KEY`], `fixed` â€” read at every\nactivation and never changed by the loop. The template this\nagent is made from must declare that name, or the registration\nis refused by the provider; the value reaches the loop through\nthe proxy's feed, never through the arguments.",
    "type": "object",
    "properties": {
      "model": {
        "description": "The model to route to, in OpenRouter's `vendor/name` form.",
        "type": "string"
      },
      "system_prompt": {
        "description": "The system prompt's text, sent as the conversation's leading\nmessage.",
        "type": [
          "string",
          "null"
        ]
      },
      "top_logprobs": {
        "description": "How many alternatives to report per token. Absent means report\nno log probabilities at all.",
        "type": [
          "integer",
          "null"
        ],
        "format": "uint64",
        "minimum": 0
      },
      "frequency_penalty": {
        "description": "Penalise tokens by how often they have appeared.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "logit_bias": {
        "description": "Per-token additive bias on the logits, keyed by token id.\n\nAn `IndexMap` rather than a `HashMap`: insertion order is\npreserved, so the same bias set serializes identically every\ntime instead of shuffling between runs.",
        "type": [
          "object",
          "null"
        ],
        "additionalProperties": {
          "type": "integer",
          "format": "int64"
        }
      },
      "max_completion_tokens": {
        "description": "Cap on generated tokens, counting reasoning tokens.",
        "type": [
          "integer",
          "null"
        ],
        "format": "uint64",
        "minimum": 0
      },
      "presence_penalty": {
        "description": "Penalise tokens for having appeared at all, regardless of count.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "stop": {
        "description": "Sequences that end generation.",
        "anyOf": [
          {
            "$ref": "#/$defs/Stop"
          },
          {
            "type": "null"
          }
        ]
      },
      "temperature": {
        "description": "Sampling temperature.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "top_p": {
        "description": "Nucleus sampling threshold.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "max_tokens": {
        "description": "Cap on generated tokens, excluding reasoning tokens.",
        "type": [
          "integer",
          "null"
        ],
        "format": "uint64",
        "minimum": 0
      },
      "min_p": {
        "description": "Minimum probability, relative to the most likely token, for a\ntoken to be considered.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "provider": {
        "description": "Which backing providers to route to.",
        "anyOf": [
          {
            "$ref": "#/$defs/Provider"
          },
          {
            "type": "null"
          }
        ]
      },
      "reasoning": {
        "description": "Reasoning configuration.",
        "anyOf": [
          {
            "$ref": "#/$defs/Reasoning"
          },
          {
            "type": "null"
          }
        ]
      },
      "repetition_penalty": {
        "description": "Multiplicative penalty on tokens already produced.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "top_a": {
        "description": "Sample only from tokens above a threshold scaled by the most\nlikely token's probability.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "top_k": {
        "description": "Sample only from the k most likely tokens.",
        "type": [
          "integer",
          "null"
        ],
        "format": "uint64",
        "minimum": 0
      },
      "verbosity": {
        "description": "How detailed responses should be.",
        "anyOf": [
          {
            "$ref": "#/$defs/Verbosity"
          },
          {
            "type": "null"
          }
        ]
      },
      "context_compression": {
        "description": "What to do when the request exceeds the context window.",
        "anyOf": [
          {
            "$ref": "#/$defs/ContextCompression"
          },
          {
            "type": "null"
          }
        ]
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
      "model"
    ],
    "$defs": {
      "Stop": {
        "description": "Sequences that end generation when produced.\n\nUntagged: one sequence goes on the wire as a bare string and\nseveral as an array, matching what providers accept without a\ncaller having to wrap a single sequence in a list.",
        "anyOf": [
          {
            "description": "One sequence.",
            "type": "string"
          },
          {
            "description": "Several. Providers typically cap this around four.",
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        ]
      },
      "Provider": {
        "description": "Which backing providers OpenRouter may route to, and in what order.\n\nOpenRouter fronts many providers for one model, and they differ in\nprice, throughput and quantization â€” so \"which model\" does not\nfully determine what answers.",
        "type": "object",
        "properties": {
          "allow_fallbacks": {
            "description": "Whether to fall back to a provider outside the preferences when\nnone of them can serve the request.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "require_parameters": {
            "description": "Only route to providers supporting every parameter sent.\nWithout this, unsupported parameters are silently dropped.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "order": {
            "description": "Preferred providers, most preferred first.",
            "type": [
              "array",
              "null"
            ],
            "items": {
              "type": "string"
            }
          },
          "only": {
            "description": "Route to these and no others.",
            "type": [
              "array",
              "null"
            ],
            "items": {
              "type": "string"
            }
          },
          "ignore": {
            "description": "Never route to these.",
            "type": [
              "array",
              "null"
            ],
            "items": {
              "type": "string"
            }
          },
          "quantizations": {
            "description": "Acceptable weight quantizations. A more aggressively quantized\nmodel is cheaper and measurably different.",
            "type": [
              "array",
              "null"
            ],
            "items": {
              "$ref": "#/$defs/ProviderQuantization"
            }
          }
        }
      },
      "ProviderQuantization": {
        "description": "A weight quantization level.",
        "oneOf": [
          {
            "description": "4-bit integer.",
            "type": "string",
            "const": "int4"
          },
          {
            "description": "8-bit integer.",
            "type": "string",
            "const": "int8"
          },
          {
            "description": "4-bit float.",
            "type": "string",
            "const": "fp4"
          },
          {
            "description": "6-bit float.",
            "type": "string",
            "const": "fp6"
          },
          {
            "description": "8-bit float.",
            "type": "string",
            "const": "fp8"
          },
          {
            "description": "16-bit float (half).",
            "type": "string",
            "const": "fp16"
          },
          {
            "description": "16-bit brain float.",
            "type": "string",
            "const": "bf16"
          },
          {
            "description": "32-bit float (full).",
            "type": "string",
            "const": "fp32"
          },
          {
            "description": "Not reported.",
            "type": "string",
            "const": "unknown"
          }
        ]
      },
      "Reasoning": {
        "description": "How much the model should reason before answering, and how much of\nthat reasoning to report.",
        "type": "object",
        "properties": {
          "enabled": {
            "description": "Whether to reason at all.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "max_tokens": {
            "description": "A token budget for reasoning. Mutually exclusive with\n[`effort`](Self::effort) on most models â€” a budget and a level\nare two ways of saying the same thing.",
            "type": [
              "integer",
              "null"
            ],
            "format": "uint64",
            "minimum": 0
          },
          "effort": {
            "description": "How hard to reason, as a level rather than a budget.",
            "anyOf": [
              {
                "$ref": "#/$defs/ReasoningEffort"
              },
              {
                "type": "null"
              }
            ]
          },
          "summary_verbosity": {
            "description": "How much of the reasoning to summarise back.",
            "anyOf": [
              {
                "$ref": "#/$defs/ReasoningSummaryVerbosity"
              },
              {
                "type": "null"
              }
            ]
          }
        }
      },
      "ReasoningEffort": {
        "description": "How hard the model should reason.",
        "oneOf": [
          {
            "type": "string",
            "enum": [
              "minimal",
              "low",
              "medium",
              "high",
              "xhigh"
            ]
          },
          {
            "description": "Do not reason.",
            "type": "string",
            "const": "none"
          }
        ]
      },
      "ReasoningSummaryVerbosity": {
        "description": "How much of the reasoning to summarise in the response.",
        "oneOf": [
          {
            "description": "Let the model decide.",
            "type": "string",
            "const": "auto"
          },
          {
            "description": "Brief.",
            "type": "string",
            "const": "concise"
          },
          {
            "description": "Thorough.",
            "type": "string",
            "const": "detailed"
          }
        ]
      },
      "Verbosity": {
        "description": "How detailed the model's responses should be. A hint â€” not every\nmodel honours it.",
        "oneOf": [
          {
            "description": "Concise.",
            "type": "string",
            "const": "low"
          },
          {
            "description": "Balanced.",
            "type": "string",
            "const": "medium"
          },
          {
            "description": "Thorough.",
            "type": "string",
            "const": "high"
          },
          {
            "description": "As detailed as the model will go.",
            "type": "string",
            "const": "max"
          }
        ]
      },
      "ContextCompression": {
        "description": "Which compression engine to enable when a request would otherwise\nexceed the model's context window. Maps onto OpenRouter's\nrequest-body `plugins` array.",
        "oneOf": [
          {
            "description": "Drops content from the middle of the conversation. The only\nengine documented today.",
            "type": "string",
            "const": "middle_out"
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
# OpenRouter

An agent container running an OpenRouter chat-completions loop, its
tools the caller's MCP servers.

Secrets: `OPENROUTER_API_KEY` (fixed), the bearer token of every
request.
