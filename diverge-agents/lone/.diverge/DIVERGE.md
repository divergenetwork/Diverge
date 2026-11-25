---
{
  "type": "agent",
  "name": "lone",
  "description": "An agent that runs a LoNE bundle on the Apple Neural Engine, calling no upstream model.",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Agent",
    "description": "An agent that runs a local LoNE bundle instead of calling an\nupstream model.\n\nThe bundle is the model: a converted Core ML directory produced by\n`lone convert`, mounted into the container at `bundle`. Every turn\nruns `lone harness` against the conversation; nothing leaves the\ncontainer. A LoNE bundle carries its own tokenizer and chat\ntemplate, so the system prompt, the sampling fields and whether\nthe model thinks are the whole of what this agent can be told.\n\n# Secrets\n\nNone: the model is on disk, so there is no credential. The\ntemplate this agent is made from need declare no name, and a name\nit does declare is one the program never asks for.\n\n# The host\n\n`lone` targets Core ML and the Apple Neural Engine, and runs on\nmacOS only. The image therefore needs a host that provides them;\non a Linux provider the harness exits saying so. See the image's\n`/HARNESS.md`.",
    "type": "object",
    "properties": {
      "bundle": {
        "description": "The converted bundle's directory inside the container.\n\nMust hold the `manifest.json` `lone convert` writes. The\nregistration checks that before the agent counts as\nregistered; a missing bundle is refused where the caller can\nsee why.",
        "type": "string"
      },
      "system_prompt": {
        "description": "The system prompt, sent as the conversation's leading message.\n\nAbsent sends none, and the bundle's own template is the whole\nof the framing.",
        "type": [
          "string",
          "null"
        ]
      },
      "max_new_tokens": {
        "description": "Cap on generated tokens for one turn, reasoning included.\n\nAbsent is the harness's 512; the bundle's context clamps it\nfurther when the conversation leaves less room.",
        "type": [
          "integer",
          "null"
        ],
        "format": "uint64",
        "minimum": 0
      },
      "temperature": {
        "description": "Sampling temperature. Absent is greedy, the harness's default.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "top_p": {
        "description": "Nucleus sampling. Absent keeps the whole distribution.",
        "type": [
          "number",
          "null"
        ],
        "format": "double"
      },
      "top_k": {
        "description": "Top-k sampling. Absent keeps every token.",
        "type": [
          "integer",
          "null"
        ],
        "format": "uint64",
        "minimum": 0
      },
      "thinking": {
        "description": "Whether the model may think before answering. Absent is the\ntemplate's own default, which is yes.",
        "type": [
          "boolean",
          "null"
        ]
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
      "bundle"
    ],
    "$defs": {
      "Template": {
        "description": "One tool container an agent's program depends on: everything the\ncaller needs to deploy it that the tool's image does not say for\nitself, as fields and not as words. A tool container depends on\nnothing.\n\nA [`Container`](crate::shared::containers::request::Container) is an\nimage, two limits, mounts and arguments. The image and the\narguments are here, member for member. The limits are the image's\nown — the `memory` and the `disk` of its\n[document](crate::shared::containers::info::Document) — and what\nthis adds to each. The mounts are not the caller's volumes — a\nvolume is named in the caller's own listing, and a tool that named\none would be dictating the caller's storage — but the AGENT's own\nfiles and directories, each a path in the agent container served\nlive at a path in the tool container: the caller serves the agent\ncontainer, a `containers::fuse_serve` of the path, and mounts the\ntool, a `fuse::mount` answered from it.\n\n# What the image says, and this does not\n\nThe image is a dependency tool's: its document's `type` is\n`dependency_tool`, and a declaration of an image of another kind —\nan agent's, or a standalone tool's — is the registration's error.\nThat document, and not this, says:\n\n1. How the tool is kept running, its\n   [`mode`](crate::shared::containers::info::Mode): lazy or eager.\n2. How far it is shared, its\n   [`scope`](crate::shared::containers::info::Scope): one per\n   agent, one per agent template, or one for all.\n3. Every secret its program could take, each by its name and what\n   it is. Which of them this tool needs is here, by name: see\n   [`secrets`](Self::secrets).\n4. The memory and the disk it needs.\n\nSo two agents that declare one image declare one kind of tool, and\nno declaration says of an image what the image does not.\n\n# The mounts are a lazy tool's of one agent\n\nA mount is the declaring agent's own path, so there must be one\nagent whose path it is, and that agent's run must be the tool's:\nonly an image whose `mode` is `lazy` and whose `scope` is\n`instance` is declared with any. A declaration that names a mount\nof an image of any other mode or scope is the registration's\nerror.\n\nWhat the tool may ask of the caller is not here: the tool's own\nprogram declares it at its registration, every time it starts, as a\nstandalone tool's does — see the `permissions` of the\n[`register`](crate::container_proxy::inside::register) answer.\n\nThere are no instructions. What a declaration cannot say — the\nprovider the tool runs on, the volumes it is given — is the\ncaller's, and the caller needs no prose to choose it. The daemon's\nown tool templates are another thing: a record the daemon keeps,\nnamed by a hash, made by a request, of a standalone tool's image;\nthis is what an agent's program declares and a caller deploys.\n\nThere is no name. A dependency is named by its ID: the lowercase\nhexadecimal SHA-256 of its [`canonical`](crate::shared::canonical)\nbytes — this template as compact JSON, absent members omitted, the\nimage's `references` absent too, every object key sorted at every\ndepth, the `arguments`, the mounts and the `secrets` included, the\nnames sorted bytewise — sixty-four characters, which the caller computes over\n[`hashed`](Self::hashed). Two declarations with one id are one\ndependency, and a list that declares one twice is refused; the same\ntemplate deployed for the same agent, on whatever provider, is the\nsame dependency, and the same template declared under the same\nscope key, by whatever agent, is the same tool. The caller labels\nthe tool's MCP tools by the id when it merges lists, and the\nprogram knows which tool serves each by the keys under `_meta`, see\n[`shared::mcp`](crate::shared::mcp).",
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
          "secrets": {
            "description": "The secrets the tool container needs, by name: all of those its\nimage's document states, or some of them, and none it does not\nstate. Required, and empty for a tool that needs none. Which of\nthem the tool's program takes, and in what mode, is its own\nanswer at its registration — see\n[`secrets`](crate::shared::containers::secrets) — and the value\nof each is resolved through the vault key the agent the tool is\nspawned for binds the same name to, fed at every activation.\nInside the hash, sorted bytewise there. A name is not empty, no\ntwo are alike, every one is a name the tool's image states, and\nevery one is a name the registering agent binds; a declaration\nthat says otherwise is the registration's error.",
            "type": "array",
            "items": {
              "type": "string"
            }
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
          "arguments",
          "secrets"
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
      "image": false,
      "audio": false,
      "file": false
    }
  },
  "structured_decisions": false
}
---

# LoNE

An agent that runs a **LoNE bundle** — LFM3 / LFM2.5 converted for Core ML and the Apple Neural
Engine — instead of calling an upstream model. The bundle is the model: every turn runs
`lone harness` over the whole conversation, and the chunks it writes are the turn.

`lone` targets Core ML and the ANE, so a turn needs **macOS**. The image installs the harness
without Core ML (the manylinux wheel of `coremltools` does not exist for arm64), so it builds and
registers anywhere; on a Linux provider a turn exits with
`lone targets the Apple Neural Engine and needs macOS` on stderr, which the loop reports as the
turn raising. Running it on a host that has the ANE is the point.

## Arguments

| member | required | meaning |
|---|---|---|
| `bundle` | yes | the converted bundle's directory **inside the container**, holding the `manifest.json` `lone convert` writes. Checked at registration; a path without it is refused |
| `system_prompt` | no | the conversation's leading system message |
| `max_new_tokens` | no | cap on generated tokens for one turn, reasoning included; absent is 512, and the bundle's context clamps it further |
| `temperature` | no | sampling temperature; absent is greedy |
| `top_p`, `top_k` | no | nucleus / top-k; absent keep the whole distribution |
| `thinking` | no | whether the model may think before answering; absent is the template's default (yes) |
| `dependencies` | no | tool containers, as in every agent-loop image |

Secrets: **none**. The model is on disk, so there is no credential, and the template this agent is
made from need declare no name.

## The bundle

Convert it where `lone` and the checkpoint live, then mount the result:

```bash
lone convert --model LFM3-1.7B --out bundles/lfm3-1.7b --ctx 2048
```

A full bundle is roughly twice the fp16 checkpoint because the decode and prefill shapes store
their weights separately; `--decode-only` halves it at the cost of slow prompts. Nothing is
installed at registration, so a bundle is mounted, not built into the image.

## Minimal template body

```json
{
  "type": "agent",
  "image": { "digest": "<digest>", "references": [{ "registry": "docker.io", "name": "diverge-local/lone" }] },
  "memory": 8589934592,
  "disk": 1073741824,
  "arguments": { "bundle": "/models/lfm3-1.7b", "temperature": 0.0, "max_new_tokens": 512 }
}
```

A template carries no mounts. Mount the bundle when the agent is made:

```json
{
  "template": "<id>",
  "name": "lone-1.7b",
  "provider": {
    "identity": { "issuer": "https://identity.diverge.network", "subject": "github.…" },
    "volume_mounts": [
      { "volume_name": "models", "volume_relative_path": ["lfm3-1.7b"], "volume_mode": "persistent", "container_path": ["models", "lfm3-1.7b"] }
    ]
  }
}
```

`memory` is a starting point: the 1.7B LUT8 stack measures ~3.5 GB on disk and the ANE runtime
holds the weights once. The mount is the bundle directory, read-only in effect: nothing writes to
it.

## The turn

The harness contract is the python image's (`diverge-agents/python/HARNESS.md`), carried to a
bundle: `{input, tools, resources}` on stdin, `{"eval": <chunks>, "stdout": ""}` as the last stdout
line. The history is replayed through the bundle's own chat template, and the generated turn is
split into `assistant_reasoning`, `assistant_text_content` and `assistant_tool_call` chunks. Tool
calls are the template's own grammar, parsed back:

    <tool_call><function=name><parameter=k>v</parameter></function></tool_call>     LFM3
    <|tool_call_start|>[name(k='v')]<|tool_call_end|>                             LFM2.5

The image's `HARNESS.md` records the settled decisions.
