//! The Diverge CLI: the daemon's client as a program.
//!
//! `diverge` dials one daemon as one account and puts the daemon's
//! requests on the command line. Which daemon and which account are
//! the document's, `cli.yaml` under the ROOT: the root is
//! `--config <dir>`, else `DIVERGE_CONFIG`, else `.diverge` under the
//! home directory, as every Diverge program finds its root; see
//! [`config`]. The document names daemons by address, each
//! with named accounts — each an `(issuer, subject)`, one of the
//! root's OpenID Connect identities — and a default account, and a
//! default daemon; with no file it is the built-in default, the daemon
//! `local` on this host at the daemon's own default port, with no
//! account. No token is in the document: every identity is under the
//! root's `identities/`, which the command `identities` signs in — by
//! a device code, the command ended once the code is printed and its
//! wait left behind, see [`detach`]; or, at Diverge's own issuer,
//! approved by the command itself and waited for, see [`ours`] —
//! lists and removes, and no command prints a token.
//!
//! [`arguments`] is the command line; [`cli`] runs it. Every request of
//! the daemon's wire is reached under the one command `run`, as a
//! subcommand under its family — `run agents create`, `run
//! standalone-tools templates list` — with the daemon and the account
//! by name, else the defaults, and the body as JSON, whose schema
//! `request-schema` beneath the request prints, and whose every answer
//! `response-schema` states; see [`run`]. The provider's, the
//! viewer's and the CLI's document in the root is each read,
//! rewritten and edited beneath its program — `provider config`,
//! `viewer config`, `cli config` — through jq, as JSON on this side;
//! see [`document`]. The daemon's is not worked on whole: five
//! settings of it are each read and written by themselves, with no
//! JSON given —
//! `daemon port get|set`, `daemon idle_seconds get|set`,
//! `daemon admins get|add|delete`, `daemon identity get|set`,
//! `daemon postgres get|set` — and each one that writes is
//! refused while the daemon runs on the root; see
//! [`cli::setting`]. Five of the provider's are too —
//! `provider port get|set`, `provider identity get|set`,
//! `provider incoming get|add|delete`,
//! `provider outgoing get|add|delete`,
//! `provider podman registries get|add|delete` — by the same rules,
//! and `provider podman storage_path get|set|delete`, which moves
//! podman's data before it writes where it is, or deletes it, and
//! `provider podman image_cache_disk get|set`, and
//! `provider containers enabled|enable|disable` with its two
//! settings, and `provider volumes enabled|enable|disable` with
//! `fixed get|add|delete` and `stores get|capacity|add|edit|delete`, an edit
//! of a store's path moving the store and a delete deleting it; see
//! [`cli::provider`]. The commands
//! `daemon start|stop` and `provider start|stop` start the program on
//! the root — its own
//! binary beside this one, spawned detached and waited for until it
//! holds the root's lock — and stop it, by one `POST /stop` on its
//! own port from this host, waited for until the lock is free; see
//! [`lifecycle`].

pub mod arguments;
pub mod cli;
pub mod config;
pub mod detach;
pub mod document;
pub mod jq;
pub mod lifecycle;
pub mod ours;
pub mod run;
