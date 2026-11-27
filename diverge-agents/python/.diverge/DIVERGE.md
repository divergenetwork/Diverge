---
{
  "type": "agent",
  "name": "python",
  "description": "An agent that runs Python source given in its arguments on every turn, with no model.",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Agent",
    "description": "An agent that runs Python instead of calling a model.\n\nNo `model` field, which is the point: nothing is sampled, so there\nis nothing to name — and no sampling parameters either. A Python\nagent occupies the same slot as a model-backed one and answers\ndeterministically, or as deterministically as its source does.\n\n# Credentials\n\nNone: there is no model, so no credential, and the script is\nhanded nothing but the conversation and the caller's lists.",
    "type": "object",
    "properties": {
      "python": {
        "description": "The source, verbatim.\n\nNever normalized — whitespace is significant in Python, so the\ntrimming applied to most string fields would change what the\ncode means. Written to disk as it is at registration, and run\nas it is every turn.",
        "type": "string"
      },
      "requirements": {
        "description": "Third-party packages the source needs: distribution name to\nversion constraint. Installed at registration, once.\n\nPrecision is the author's statement of intent: a loose\nspecifier says \"track updates\", an exact one says \"freeze\nthis\". A map rather than a list of requirement lines, so one\npackage cannot appear twice with constraints that contradict\neach other. Ordered, so the same set always serializes — and\ninstalls — identically instead of shuffling between runs.\n\nOne constraint per package, which is what the shape costs. A\ncompound range (`>= 2, < 3`) and an environment marker\n(`; python_version < \"3.11\"`) are both unrepresentable — the\nfirst needs two constraints for one name, the second needs a\nplace to put the marker. Both are legal in a\n`requirements.txt` and neither survives here.",
        "type": "object",
        "additionalProperties": {
          "$ref": "#/$defs/Version"
        }
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
      "python"
    ],
    "$defs": {
      "Version": {
        "description": "A constraint on which version of a package satisfies a\nrequirement.\n\nSplit into an operator and a version rather than kept as one\nstring, so the operator is a closed set a consumer matches on\ninstead of a prefix it has to parse off the front. `\">==2.0\"` is\nnot representable.\n\nThere is no \"any version\" — an operator is always required. Say\n`>= 0` and mean it, rather than leaving a consumer to decide what\nan absent constraint implies.",
        "type": "object",
        "properties": {
          "operator": {
            "description": "How to compare.",
            "$ref": "#/$defs/Operator"
          },
          "version": {
            "description": "What to compare against — a PEP 440 version, without the\noperator. `\"2.31.0\"`, not `\">=2.31.0\"`.\n\nLeft a string rather than parsed into components: PEP 440\nversions carry epochs, pre/post/dev segments and local\nidentifiers (`1!2.0.0rc1.post2+ubuntu.1`), and a structure that\nmodelled all of it would be a version parser in a wire type.",
            "type": "string"
          }
        },
        "required": [
          "operator",
          "version"
        ]
      },
      "Operator": {
        "description": "How a [`Version`](super::Version) compares, as PEP 440 defines\nthem.\n\nA closed set, so it is an enum rather than a string: an operator\npip does not understand fails to deserialize instead of failing\nat install time.\n\nSeven of PEP 440's eight. `===`, arbitrary equality, is omitted —\nit exists for versions that are not PEP 440 versions, and PyPI\nenforces PEP 440 on upload, so nothing reachable from the default\nindex can need it.",
        "oneOf": [
          {
            "description": "`==` — this version. The default, because freezing is the\nsafer thing to do by accident.",
            "type": "string",
            "const": "=="
          },
          {
            "description": "`!=` — anything but this version.",
            "type": "string",
            "const": "!="
          },
          {
            "description": "`>=` — this version or newer. The usual way to say \"track\nupdates\".",
            "type": "string",
            "const": ">="
          },
          {
            "description": "`>` — strictly newer.",
            "type": "string",
            "const": ">"
          },
          {
            "description": "`<=` — this version or older.",
            "type": "string",
            "const": "<="
          },
          {
            "description": "`<` — strictly older.",
            "type": "string",
            "const": "<"
          },
          {
            "description": "`~=` — compatible release. Newer, but not past the last\ncomponent: `~= 2.31.0` allows `2.31.4` and refuses `2.32.0`.",
            "type": "string",
            "const": "~="
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
      "audio": true,
      "file": true
    }
  },
  "structured_decisions": false
}
---
# Python

An agent container running the arguments' Python source on every
turn against the conversation and the caller's tool and resource
lists; no model.

Credentials: none.
