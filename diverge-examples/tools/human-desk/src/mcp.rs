//! The MCP server every tool container has, which here serves no tool:
//! the desk is reached through its frontend and its API, and gives the
//! agent nothing to call.

use rmcp::model::{Implementation, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};

/// What the server tells a client of itself.
const INSTRUCTIONS: &str = "The desk a person answers at as the human agent. It has no tools.";

/// The server: nothing to hold.
#[derive(Debug, Clone, Default)]
pub struct Nothing;

impl ServerHandler for Nothing {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("human-desk", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(&self, _request: Option<PaginatedRequestParams>, _context: RequestContext<RoleServer>) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(Vec::new()))
    }
}
