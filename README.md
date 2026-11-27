# Diverge

**The Superintelligence Network.**

Diverge is a system that runs superintelligence agents, and the tools those agents use, in containers on machines the operator selects, and that connects those machines into one network. Each agent is one container. Each agent has its own set of tools, its own permissions and its own log. An agent communicates with other agents, with persons, and with agents on machines operated by other parties, and the system records every such communication.

## Installation by an agent

The following prompt instructs a superintelligence agent to install Diverge on the machine it runs on and to assist the operator in bringing the system into use. The operator gives the prompt to the agent in full. It is a single line, so that it is copied whole.

```text
Install Diverge on this machine, and then assist me in bringing it into use. (1) Determine this machine's operating system, which is one of `linux`, `macos` and `windows`, and its architecture, which is one of `x86_64` and `aarch64`. (2) Download https://releases.diverge.network/latest/<os>/<arch>, substituting the two values. The file is a `.tar.gz` archive on Linux and macOS and a `.zip` archive on Windows. The archive contains exactly one folder, and that folder contains the programs. (3) Create the directory `~/.diverge/bin` if it does not exist. Place every file of that folder directly in `~/.diverge/bin`, with no folder between them. On Linux and macOS, confirm that each file is executable. Do not create, read or modify anything else under `~/.diverge`. (4) Add `~/.diverge/bin` to my PATH for this session and for future sessions. Run `diverge --version` and confirm that it prints a version. (5) Download https://releases.diverge.network/latest/skills. The file is a `.zip` archive that contains one folder for each skill. Extract it into the directory from which you load skills, so that each skill's folder is directly in that directory. If you do not know that directory, ask me. (6) Load the skill named `diverge`. It states which further skill to load for each task. Follow it from this point onward, and load each skill before you perform the task it covers. (7) Assist me, in this order: sign me in; configure and start the provider; configure and start the daemon; and make my account an administrator of the daemon. Ask me for each decision that is mine to make, and do not choose on my behalf where a skill states that the choice is the operator's.
```

## Manual installation

1. **Programs.** Download the archive for the platform from the table below. The archive contains exactly one folder, which contains the programs. Place every file of that folder directly in `~/.diverge/bin`, creating that directory if it does not exist.
2. **Path.** Add `~/.diverge/bin` to the `PATH` environment variable. The command `diverge --version` prints the installed version when the installation is correct.
3. **Skills.** Download the skills archive from the link below the table. Extract it into the skills directory of the agent that is to operate Diverge, so that each skill's folder is directly in that directory.
4. **First use.** Instruct the agent to load the skill named `diverge`. That skill is the index of the others and states which skill governs each task, beginning with signing in and starting the provider and the daemon.

## Download

Each archive contains every program for one platform. Each link below resolves to the most recent release.

| | x86_64 | ARM64 |
|---|---|---|
| **Linux** | [download](https://releases.diverge.network/latest/linux/x86_64) | [download](https://releases.diverge.network/latest/linux/aarch64) |
| **macOS** | [download](https://releases.diverge.network/latest/macos/x86_64) | [download](https://releases.diverge.network/latest/macos/aarch64) |
| **Windows** | [download](https://releases.diverge.network/latest/windows/x86_64) | [download](https://releases.diverge.network/latest/windows/aarch64) |

**Skills.** The skills are the instructions by which a superintelligence agent operates Diverge on the operator's behalf. They are distributed as one archive: [download](https://releases.diverge.network/latest/skills). The archive contains one folder for each skill and is extracted into the skills directory of the agent that is to use them.

Every released version is listed at [releases.diverge.network](https://releases.diverge.network).

After installation, the command `diverge update <skills directory>` replaces the programs and the skills with those of the next version in a single operation.

## Description of the system

**Containers.** An agent is a container that receives messages and answers them. The program inside an agent's container is a model harness, a script, or a web page at which a person answers. A tool is a container that serves the [Model Context Protocol](https://modelcontextprotocol.io). Neither an agent nor a tool is installed on the host system. A container is started when there is work for it and is stopped when there is none.

**Images.** Each agent and each tool is an OCI image, identified by its digest. An agent or a tool is put into use by one command, and no file of the host system is modified by doing so. An agent or a tool is removed by deleting it, and no part of it remains on the host system after its removal. Everything that an agent does, it does inside its own container, within the memory and disk allotted to that container and the volumes mounted into it.

**Tool control.** An agent is able to perform exactly those actions that its tools permit, and no others. Different agents are given different tools. One agent reads a codebase; a second agent holds the credential that deploys it; a third agent is able only to send messages. A system is therefore composed of specialised agents, each of which holds the minimum set of tools its task requires, and not of a single agent that holds every tool.

**Communication between agents.** The capacity of one agent to send a message to another is not a property of the agent. It is a tool, which is served to the agents to which it is attached and which operates through the daemon. Whether one agent is able to communicate with another is accordingly determined by which tools each agent holds. Each message so sent is recorded by the daemon as an event.

**Providers.** A provider runs containers, holds volumes and stores images. A daemon operates with every provider connected to it and places each container on a provider that is able to run that container. Any program that conforms to the [Diverge Provider Protocol](https://protocol.diverge.network) is a provider. The provider in this repository runs containers by means of podman. A party may implement a provider of its own.

**The network.** Two daemons that are connected to the same provider are able to reach one another through that provider. A daemon is able to have a request served by another daemon, to mount a volume held by another daemon, and to supply to its own agents a tool that runs on a machine operated by another party. The provider carries the connection between the two daemons and is not able to read its contents.

**Accounts, roles and grants.** Every party that connects to a daemon is an account. An account is a person, another daemon, or a tool acting in its own name. An account holds one or more roles. A role is a list of grants. A grant names an action and the extent of the records the action may be applied to: every record of a kind, specified records, or the records that bear a specified tag. No action is permitted that no grant permits.

**Identity.** Every connection begins with a mutual handshake between two identities, each of which is signed in at an OpenID Connect issuer. Each party proves possession of the private key to which its token is bound. The connection is encrypted between the two parties from the completion of the handshake. A token obtained by a third party cannot be used by that party, because the third party does not hold the private key.

**Logs and audit.** The daemon retains, for every agent, the entirety of its conversation. It retains, for every tool, every call made to that tool, the identity of the caller, and the answer returned. It retains, for every account, every request that the account made of the daemon, together with whether the request was permitted or refused. It retains a graph of which party sent what to which party. From these records it is possible to establish what occurred, which party requested it, and whether that party was permitted to request it.

**Credentials.** A credential that the program of an image requires is a member of the arguments of that image, and is stated in a template. The arguments of a template are returned only when the request states that they are to be returned, and only to a party that holds the grant to see them. The daemon's record of each request has the arguments of every template removed from it.

**Requests.** An agent, a tool or a person is able to address a question to specified accounts and tools and to wait for exactly one answer. A request is the means by which an agent obtains a decision from a person.

## The programs

| Program | Function |
|---|---|
| `diverge` | The command-line program. Each setting and each request is one command, which accepts and returns JSON. |
| `diverge-daemon` | The daemon. It holds accounts, roles, agents, tools, templates, events and logs in a Postgres database of its own, and it starts and stops containers on providers. |
| `diverge-provider` | The provider. It runs containers by means of podman, serves volumes and maintains an image store. |
| `diverge-viewer` | A desktop application. It displays the current state of one daemon: every agent, tool, account, volume and provider, the relations between them, and their logs, files and consoles. |
| `diverge-container-proxy` | The program that the provider places in every container. It carries messages, tool calls and database connections between the provider and the program inside the container. |
| `diverge-postgres` | The supervisor of the daemon's Postgres database. |

## Documentation

- **Installation from this repository:** [reports/SETUP_INSTRUCTIONS_1.md](reports/SETUP_INSTRUCTIONS_1.md).
- **Operation of an installed system:** the skills in [diverge-skills](diverge-skills), beginning with [diverge-skills/diverge/SKILL.md](diverge-skills/diverge/SKILL.md). The skills are written to be read by an agent and constitute the operating manual.
- **Implementation of a provider:** [protocol.diverge.network](https://protocol.diverge.network), which is the specification of the protocol by which a daemon communicates with a provider.
- **Development against the system:** the software development kits, [`diverge-sdk`](https://crates.io/crates/diverge-sdk) for Rust and [`@diverge/sdk`](https://www.npmjs.com/package/@diverge/sdk) for TypeScript.

## Contents of this repository

| Folder | Contents |
|---|---|
| `diverge-daemon` | The daemon. |
| `diverge-provider` | The provider. |
| `diverge-container-proxy` | The program placed in every container. |
| `diverge-postgres` | The supervisor of the daemon's Postgres database. |
| `diverge-cli` | The `diverge` command-line program. |
| `diverge-viewer` | The desktop application. |
| `diverge-sdk-rs`, `diverge-sdk-js` | The software development kits. |
| `diverge-agents` | The agent images: `cc`, `codex`, `hermes`, `lone`, `openrouter` and `python`. |
| `diverge-tools` | The tool images: `identity-diverge-network`. |
| `diverge-examples` | Example images: one agent that a person answers as, and tools for chat, for mail between agents, for memory, for a task board and for alarms. |
| `diverge-skills` | The skills. |
| `diverge-provider-web` | The specification of the provider protocol, as published at [protocol.diverge.network](https://protocol.diverge.network). |

## License

This software is licensed under the MIT License. The terms are stated in [LICENSE](LICENSE).
