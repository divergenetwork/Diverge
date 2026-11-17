//! One run of the program.

use super::Error;
use crate::arguments::{Arguments, Command, Daemon, Own, Server, Viewer};
use crate::config;
use crate::lifecycle::{self, Program};

/// Run `diverge` as told: the root found; then, for `run`, `cli.yaml`
/// read, the daemon and the account selected, and the request put —
/// see [`run`](crate::run); for `identities`, the root's identities
/// signed in, listed or removed — see [`identities`](super::identities);
/// for `daemon` and `provider`, the program started on the root or
/// stopped — see [`lifecycle`]; for `daemon port`,
/// `daemon idle_seconds`, `daemon admins`, `daemon identity` and
/// `daemon postgres`, the one setting read or written — see
/// [`setting`](super::setting); for `provider port`,
/// `provider identity`, `provider incoming`, `provider outgoing` and
/// `provider podman registries`, likewise — see [`provider`](super::provider);
/// for `viewer daemon` and for `cli daemons` and `cli default`, the
/// one setting of the viewer's document or of the CLI's own — see
/// [`viewer`](super::viewer) and [`own`](super::own).
/// With no command the root is found and the run ends.
pub async fn run(arguments: Arguments) -> Result<(), Error> {
    let root = config::root(arguments.config).await?;
    match arguments.command {
        None => Ok(()),
        Some(Command::Run(run)) => {
            let config = config::load(&root).await?;
            let (daemon, account) = (run.daemon.clone(), run.account.clone());
            let selected = config::select(&config, daemon.as_deref(), account.as_deref())?;
            crate::run::run(&root, &selected, run).await?;
            Ok(())
        }
        Some(Command::Identities(command)) => super::identities::run(&root, command).await,
        Some(Command::IdentityDivergeNetwork(command)) => super::identity_diverge_network::run(&root, command).await,
        Some(Command::Daemon(Daemon::Lifecycle(action))) => {
            Ok(lifecycle::run(&root, Program::Daemon, action).await?)
        }
        Some(Command::Daemon(Daemon::Port(action))) => super::setting::port(&root, action).await,
        Some(Command::Daemon(Daemon::IdleSeconds(action))) => {
            super::setting::idle_seconds(&root, action).await
        }
        Some(Command::Daemon(Daemon::Admins(action))) => super::setting::admins(&root, action).await,
        Some(Command::Daemon(Daemon::Identity(action))) => {
            super::setting::identity(&root, action).await
        }
        Some(Command::Daemon(Daemon::Postgres(action))) => {
            super::setting::postgres(&root, action).await
        }
        Some(Command::Provider(Server::Lifecycle(action))) => {
            Ok(lifecycle::run(&root, Program::Provider, action).await?)
        }
        Some(Command::Provider(Server::Port(action))) => super::provider::port(&root, action).await,
        Some(Command::Provider(Server::Identity(action))) => {
            super::provider::identity(&root, action).await
        }
        Some(Command::Provider(Server::Incoming(action))) => {
            super::provider::incoming(&root, action).await
        }
        Some(Command::Provider(Server::Outgoing(action))) => {
            super::provider::outgoing(&root, action).await
        }
        Some(Command::Provider(Server::Podman(action))) => {
            super::provider::podman(&root, action).await
        }
        Some(Command::Provider(Server::Containers(action))) => {
            super::provider::containers(&root, action).await
        }
        Some(Command::Provider(Server::Volumes(action))) => {
            super::provider::volumes(&root, action).await
        }
        Some(Command::Provider(Server::Images(action))) => {
            super::provider::images(&root, action).await
        }
        Some(Command::Provider(Server::Messages(action))) => {
            super::provider::messages(&root, action).await
        }
        Some(Command::Viewer(Viewer::Daemon(action))) => super::viewer::daemon(&root, action).await,
        Some(Command::Cli(Own::Daemons(action))) => super::own::daemons(&root, action).await,
        Some(Command::Cli(Own::Default(action))) => super::own::default(&root, action).await,
    }
}
