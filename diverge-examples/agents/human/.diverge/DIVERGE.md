---
{
  "type": "agent",
  "name": "human",
  "description": "An agent with no model: a person answers as it, at its desk, a tool of its own.",
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Arguments",
    "description": "The arguments a human agent is made with.\n\nThe agent is whoever answers at its desk, and its tools are\nwhichever are attached to it. The desk is a tool container of its\nown — the `human-desk` image — which the agent declares as its one\ndependency, so the agent must be told which image that is. Any\nother member is refused, so a misspelled setting is never silently\nignored.",
    "type": "object",
    "properties": {
      "desk": {
        "description": "The `human-desk` image the agent's desk is made from: its\ndigest, and the references under which its bytes may be\nfetched. The agent declares it as a dependency tool with no\narguments, no mounts and nothing beyond what the image needs.",
        "$ref": "#/$defs/Image"
      }
    },
    "additionalProperties": false,
    "required": [
      "desk"
    ],
    "$defs": {
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
# Human

An agent with no model: **you are the agent.** When someone messages it,
the message waits at its desk until you answer. You write the agent's
replies, have it call its tools, and end the turn, which sends
everything you did back as the agent's turn.

The agent serves no page of its own: an agent has no frontend. Its desk
is a second container, the **`human-desk`** tool, which this agent
declares as its one dependency and reaches over the desk's API. You
answer at the desk's frontend.

- **A run** opens one stream from the agent to its desk and hands the
  run's messages over. Every chunk of the turn comes back down that
  stream, and those chunks are the agent's run.
- **Tool calls are the agent's.** You choose the tool and its arguments
  at the desk; the desk asks the agent, the agent makes the call
  through its own tools, and the result goes back to the desk.
- **A message that arrives while you are at it** goes to the desk at
  once and is part of the same turn.
- **Nothing times out.** The agent waits on you for as long as it takes.

Arguments: `{"desk": {"digest": "sha256:…"}}`, the `human-desk` image,
with its `references` where its bytes are to be fetched from a registry.

## What it asks of the daemon

One grant, declared at its registration: `api` over the dependency
tools attached to it, which is its desk and nothing else.

```json
{"dependency_tools": {"actions": ["api"], "within": "attached"}}
```

## Running it

1. Have both images where the provider can find them: this one and
   `human-desk`.
2. Make an agent template of this image (`agents templates create`)
   whose `arguments` name the `human-desk` image's digest, and an agent
   of the template (`agents create`). Attach the tools you want to call
   as it.
3. Message the agent. Its desk is deployed with it.
4. Open the desk's frontend: the agent's dependency tool, in
   diverge-viewer's frontend tab, or `dependency-tools frontend` with
   the dependency template's id and the agent as its owner.
