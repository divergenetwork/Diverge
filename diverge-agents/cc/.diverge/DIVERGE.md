---
{
  "type": "agent",
  "name": "cc",
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
    "description": "An agent running against Claude Code.\n\nFar fewer knobs than OpenRouter's, and not because anything is\nmissing: Claude Code is an agent harness in its own right, so\nthe sampling decisions OpenRouter exposes are made inside it.\nWhat it does expose is its tools: [`tools`](Self::tools) is the\nexact set of built-ins the model is given, every switch stated,\npassed to Claude Code as `--tools`;\n[`dependencies`](Self::dependencies) is the other kind, the tool\ncontainers the agent asks the caller to run for it.\n\n# Secrets\n\nNone: Claude Code logs in by itself, and no argument names a\ncredential. The template this agent is made from need declare no\nname, and a name it does declare is one the program never asks\nfor.",
    "type": "object",
    "properties": {
      "model": {
        "description": "The model to run.",
        "type": "string"
      },
      "tools": {
        "description": "The switchable built-in tools — each absent or `false` is\noff, and the whole structure absent is every tool off. The\nMCP tools and `Skill` are on regardless; see [`Tools`].",
        "$ref": "#/$defs/Tools",
        "default": {}
      },
      "thinking": {
        "description": "Whether to think before answering.",
        "type": [
          "boolean",
          "null"
        ]
      },
      "effort": {
        "description": "How much effort to spend thinking. Meaningful only with\n[`thinking`](Self::thinking) on.",
        "anyOf": [
          {
            "$ref": "#/$defs/Effort"
          },
          {
            "type": "null"
          }
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
      "model"
    ],
    "$defs": {
      "Tools": {
        "description": "The built-in tools Claude Code is given, every one stated.\n\nClaude Code's `--tools` flag takes the exact set of built-ins the\nmodel may call, and this is that set: a switch per tool, each an\n`Option<bool>` where ABSENT IS OFF — the same as `false` — so\nnothing is on by omission and a request names only what it\nturns on. The harness renders the `true` switches as the flag's\nlist, in this order, after the always-on names.\n\nAlways on, no switch: the `mcp__*` tools — the caller's own\ntools, served through the proxy — are not built-ins and are\npresent whatever `--tools` says: the tool channel is what the\nprotocol is built around. Beside them the harness always lists\nthe four MCP resource built-ins (`ListMcpResources`,\n`ReadMcpResource`, `ReadMcpResourceDir`, `RefreshMcpTools` — the\ncaller's resources, through the same proxy), `Skill` (skills are\nthe directories mounted under `~/.claude/skills`; the tool is\noffered whether or not any are, and a call names a skill that is\nnot there fails as \"unknown skill\"), and `ToolSearch`, by which\nthe model loads deferred tools' schemas.\n\nAlways off, no switch: `LSP` (nothing in the image installs a\nlanguage server). Nor does a switch exist for what cannot work\nin the container, which the harness never lists:\n`AskUserQuestion`, `EnterPlanMode`,\n`ExitPlanMode`, `ProposeGoal` and `ProposeSkills` (a call waits\non a user who is not there); `CronCreate`, `CronDelete`,\n`CronList` and `ScheduleWakeup` (the container does not outlive\nthe run); `Artifact`, `Projects`, `ClaudeDesign`, `RemoteTrigger`,\n`PushNotification`, `SendFeedback`, `ReadNotifications`,\n`ShowOnboardingRolePicker` and `EndConversation` (claude.ai\naccount and host features). Nor for the legacy names the binary\nstill answers to (`MultiEdit`, `BashOutput`, `KillShell`): they\nare the old spellings of switches here.\n\nWhether the pinned build honors every name here under `-p` with\nstream-json input is a live-run check, not a promise of this\ntype; the names are as the runtime spells them.",
        "type": "object",
        "properties": {
          "bash": {
            "description": "`Bash`: runs a command, with an optional timeout and\noptionally in the background.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "read": {
            "description": "`Read`: reads a file by absolute path, with `offset` and `limit`\nfor large files and `pages` for PDFs.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "write": {
            "description": "`Write`: writes a whole file, overwriting.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "edit": {
            "description": "`Edit`: exact-string replacement in a file.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "notebook_edit": {
            "description": "`NotebookEdit`: replaces, inserts or deletes one Jupyter cell.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "glob": {
            "description": "`Glob`: file pattern matching.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "grep": {
            "description": "`Grep`: ripgrep over the workspace (ripgrep ships with the\nbinary).",
            "type": [
              "boolean",
              "null"
            ]
          },
          "task": {
            "description": "`Task` (also spelled `Agent`): launches a sub-agent. Its chunks\nreach the run's stream attributed by `parent_tool_call_id`.\nLocal sub-agents only: `isolation: remote` needs a cloud\nenvironment the container has no account for.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "send_message": {
            "description": "`SendMessage`: messages a running named agent.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "list_agents": {
            "description": "`ListAgents`: lists addressable agents.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "task_output": {
            "description": "`TaskOutput`: reads the output of a background task, blocking\nor not.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "task_stop": {
            "description": "`TaskStop`: stops a background task by id.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "monitor": {
            "description": "`Monitor`: watches a command's stdout lines or a WebSocket's\nframes as events.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "workflow": {
            "description": "`Workflow`: runs a deterministic multi-agent orchestration\nscript.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "todo_write": {
            "description": "`TodoWrite`: the session's structured todo list.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "task_create": {
            "description": "`TaskCreate`: creates an entry on the session's task board.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "task_get": {
            "description": "`TaskGet`: reads one task board entry.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "task_update": {
            "description": "`TaskUpdate`: updates one task board entry.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "task_list": {
            "description": "`TaskList`: lists the task board.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "report_findings": {
            "description": "`ReportFindings`: reports code-review findings as a typed list.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "repl": {
            "description": "`REPL`: runs JavaScript with persistent state and top-level\nawait.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "web_fetch": {
            "description": "`WebFetch`: fetches a URL and runs a prompt over the content.\nNeeds outbound network from the container.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "web_search": {
            "description": "`WebSearch`: searches the web, with domain allow and block\nlists. Needs outbound network from the container.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "enter_worktree": {
            "description": "`EnterWorktree`: creates or switches into a git worktree. Needs\nthe workspace to be a git repository.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "exit_worktree": {
            "description": "`ExitWorktree`: leaves the worktree, keeping or removing it.",
            "type": [
              "boolean",
              "null"
            ]
          },
          "sleep": {
            "description": "`Sleep`: waits.",
            "type": [
              "boolean",
              "null"
            ]
          }
        }
      },
      "Effort": {
        "description": "How much effort the model should spend before answering.",
        "oneOf": [
          {
            "description": "Least.",
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
            "description": "Above high, below max.",
            "type": "string",
            "const": "xhigh"
          },
          {
            "description": "As much as the model will spend.",
            "type": "string",
            "const": "max"
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
              "mode": {
                "type": "string",
                "const": "lazy"
              },
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
              "mode": {
                "type": "string",
                "const": "eager"
              },
              "scope": {
                "description": "How far the tool is shared, and its database with it: see\n[`Scope`]. Required.",
                "$ref": "#/$defs/Scope"
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
  }
}
---
# Claude Code

An agent container running a Claude Code agent: the harness installs
the pinned Claude Code, runs it as a subprocess, and serves the loop.

Secrets: none. Claude Code logs in by itself.
