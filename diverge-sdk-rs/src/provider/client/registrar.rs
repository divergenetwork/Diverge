//! What a container declares at registration, served by the caller.

use std::future::Future;

use crate::shared::containers::dependencies::Template;
use crate::shared::error::Error;
use crate::shared::permission::Grant;

/// The caller's answer to a provider asking it to register a
/// container: deploy the dependencies its program declared, and hold
/// it to the permissions it declared.
///
/// Each dependency is a tool container the caller runs — a
/// `containers::tools::run` from the template's image, limits and
/// arguments, the agent's own paths served into it as the template's
/// mounts say, on the database and under the grants it names — and
/// serves to the container under the template's id among the MCP
/// servers it answers the container's tool calls with. Each permission is a
/// grant the container's program holds over the caller for the run,
/// an agent's or a tool's, and the whole of what it may ask. The provider asks
/// once per run, before the id, and only when the container declared
/// at least one of either; the run goes on to its id on `Ok` and ends
/// on `Err`, the container stopped, with the error carried to the
/// caller as the caller's policy.
pub trait Registrar: Send + Sync {
    /// Run every dependency for the container running under `id` on
    /// the provider that asks — the id its run will answer, told
    /// first so the caller can serve the container's paths into the
    /// dependencies — and hold the container to `permissions`; or say
    /// why not.
    fn register(
        &self,
        id: String,
        dependencies: Vec<Template>,
        permissions: Vec<Grant>,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}
