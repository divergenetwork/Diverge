//! The container registered: its dependencies deployed, its
//! permissions held.

use diverge_sdk::provider::client::Registrar;
use diverge_sdk::shared::containers::dependencies::Template;
use diverge_sdk::shared::error::Error;
use diverge_sdk::shared::permission::Grant;

use super::Answerer;
use crate::containers::deploy;

/// The one ask a provider makes before the container's id is out, the
/// id told here, answered in order:
///
/// 1. The permissions a tool's program declared kept on the answerer:
///    what its `/daemon` connections are served under for the run, and
///    what the start records on the tool's row. An agent's — which the
///    provider never sends — are dropped, since an agent asks the
///    daemon nothing.
/// 2. Every dependency deployed as [`deploy`](crate::containers::deploy)
///    states, where the program declared any; the first that cannot be
///    is the run's error. A tool container declares none, and one that
///    does is refused there.
impl Registrar for Answerer {
    async fn register(&self, id: String, dependencies: Vec<Template>, permissions: Vec<Grant>) -> Result<(), Error> {
        if self.tool.is_some() {
            *self.permissions.lock().await = permissions;
        }
        if !dependencies.is_empty() {
            deploy::deploy(self, id, dependencies).await?;
        }
        Ok(())
    }
}
