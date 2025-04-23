mod stdio;
mod sse;

use anyhow::Result;
use rmcp::{service::Peer, RoleClient};
use crate::config::McpServerConfig;

/// Create an MCP client based on the server configuration
pub async fn create_mcp_client(config: &McpServerConfig) -> Result<Peer<RoleClient>> {
    match config {
        McpServerConfig::Stdio { .. } => {
            stdio::create_stdio_client(config).await
        },
        McpServerConfig::Sse { .. } => {
            sse::create_sse_client(config).await
        }
    }
}
