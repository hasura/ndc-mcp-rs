mod stdio;
mod sse;

use anyhow::Result;
use rmcp::{service::RunningService, RoleClient};
use crate::config::McpServerConfig;

/// Create an MCP client based on the server configuration
pub async fn create_mcp_client(config: &McpServerConfig) -> Result<RunningService<RoleClient, ()>> {
    match config {
        McpServerConfig::Stdio(stdio_config) => {
            stdio::create_stdio_client(stdio_config).await
        },
        McpServerConfig::Sse(sse_config) => {
            sse::create_sse_client(sse_config).await
        }
    }
}
