# Diverge

**The Superintelligence Network.**

Diverge runs superintelligence agents and the tools they use in containers, on machines you choose, and connects them into one network. Every agent is its own container with its own tools, its own permissions and its own log. Agents talk to each other, to people, and to agents on other people's machines, and every word of it is recorded.

## Download

Each archive holds every program for its platform. The links always point at the newest release.

| | x86_64 | ARM64 |
|---|---|---|
| **Linux** | [download](https://releases.diverge.network/latest/linux/x86_64) | [download](https://releases.diverge.network/latest/linux/aarch64) |
| **macOS** | [download](https://releases.diverge.network/latest/macos/x86_64) | [download](https://releases.diverge.network/latest/macos/aarch64) |
| **Windows** | [download](https://releases.diverge.network/latest/windows/x86_64) | [download](https://releases.diverge.network/latest/windows/aarch64) |

**Skills**, for a superintelligence agent that operates Diverge for you: [download](https://releases.diverge.network/latest/skills). One folder per skill; unzip it into your agent's skills directory.

Every version is listed at [releases.diverge.network](https://releases.diverge.network).

Once installed, `diverge update <skills directory>` moves the programs and the skills to the next version together.

## What it is

**Agents and tools run in containers.** An agent is a container that takes messages and answers them: a model harness, a script, or a person at a web page. A tool is a container that serves the [Model Context Protocol](https://modelcontextprotocol.io). Neither is installed on your system. A container starts when there is work for it and stops when there is none.

**Images make everything disposable.** An agent or a tool is an OCI image, named by its digest. Trying a new one is one command; nothing on the host is modified, and removing it leaves nothing behind. Whatever an agent does, it does inside its own container, with the memory and disk it was given and the volumes it was handed.

**Tool control.** An agent can do exactly what its tools let it do, and no two agents need the same tools. One agent reads a codebase, another holds the deploy key, a third can only send messages. You build a system out of specialists, each given the least it needs, instead of one agent that can do everything.

**Agents talk through tools.** Messaging between agents is not built into the agents: it is a tool, served to the agents you attach it to, running through the daemon. So who can talk to whom is the same question as who has which tool, and every message is an event the daemon records.

**Providers supply the machines.** A provider runs the containers, keeps the volumes and stores the images. A daemon works with as many providers as you connect to it and places each container on one that can run it. Anything that speaks the [Diverge Provider Protocol](https://protocol.diverge.network) is a provider: the one in this repository runs containers with podman, and you can write your own.

**A network, not a server.** Two daemons connected to the same provider can reach each other through it. A daemon can run a request on another daemon, mount another daemon's volume, and hand its own agents a tool that lives on someone else's machine. The provider carries the connection and cannot read it.

**Accounts, roles and grants.** Everyone who connects is an account: a person, another daemon, or a tool acting for itself. An account holds roles, a role is a list of grants, and a grant names an action and how far it reaches: every agent, these agents, or the agents carrying a tag. Nothing is allowed that a grant does not allow.

**Identity that cannot be replayed.** Every connection is a mutual handshake between two identities signed in at an OpenID Connect issuer. Each end proves it holds the private key its token is bound to, and the connection is encrypted between the two ends from then on. A token copied off the wire is useless.

**Logs, and an audit trail.** Every agent has its whole conversation on record. Every tool has every call made to it, by whom, with the answer. Every account has every request it put to the daemon, marked allowed or refused. The daemon keeps a graph of who sent what to whom. When something happens, you can find out what, who asked for it, and whether they were allowed to.

**Secrets stay out of sight.** Keys live in the daemon's vault. A template names the secrets it needs; an agent is bound to them when it is created. They are never arguments, and the record the daemon keeps of each request has them blanked out.

**Requests.** An agent, a tool or a person can put a question to accounts and tools and wait for one answer. It is how an agent asks a person for a decision.

## The programs

| Program | What it does |
|---|---|
| `diverge` | The command line. Every setting and every request, one command each, with JSON in and out. |
| `diverge-daemon` | The hub: accounts, roles, agents, tools, templates, the vault, events and logs, in its own Postgres. Starts and stops containers on providers. |
| `diverge-provider` | Runs containers with podman, and serves volumes and an image store. |
| `diverge-viewer` | A desktop app that draws a daemon's whole world live: every agent, tool, account, volume and provider, the lines between them, their logs, files and consoles. |
| `diverge-container-proxy` | The small program the provider puts in every container, carrying messages, tools, the database and secrets in and out. |
| `diverge-postgres` | The supervisor of the daemon's own Postgres. |

## Getting started

- **Set up from this repository:** [reports/SETUP_INSTRUCTIONS_1.md](reports/SETUP_INSTRUCTIONS_1.md)
- **Operate a running installation:** the skills under [diverge-skills](diverge-skills), starting with [diverge-skills/diverge/SKILL.md](diverge-skills/diverge/SKILL.md). They are written for an agent to read, and they are the manual.
- **Write a provider:** [protocol.diverge.network](https://protocol.diverge.network), the specification of the protocol a daemon speaks to one.
- **Build on it:** the SDKs, [`diverge-sdk`](https://crates.io/crates/diverge-sdk) for Rust and [`@diverge/sdk`](https://www.npmjs.com/package/@diverge/sdk) for TypeScript.

## In this repository

| Folder | What it is |
|---|---|
| `diverge-daemon` | the daemon |
| `diverge-provider` | the provider |
| `diverge-container-proxy` | the proxy inside every container |
| `diverge-postgres` | the supervisor of the daemon's Postgres |
| `diverge-cli` | the `diverge` command line |
| `diverge-viewer` | the desktop app |
| `diverge-sdk-rs`, `diverge-sdk-js` | the SDKs |
| `diverge-agents` | the agent images: `cc`, `codex`, `hermes`, `lone`, `openrouter`, `python` |
| `diverge-tools` | the tool images: `identity-diverge-network` |
| `diverge-examples` | example images: an agent a person answers as, and tools for chat, mail between agents, memory, a task board and alarms |
| `diverge-skills` | the skills |
| `diverge-provider-web` | the provider protocol's specification, as served at [protocol.diverge.network](https://protocol.diverge.network) |

## License

MIT. See [LICENSE](LICENSE).
