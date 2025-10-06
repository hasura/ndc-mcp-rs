use anyhow::{anyhow, Result};
use rmcp::{
    service::RunningService,
    transport::streamable_http_client::{
        StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
    },
    RoleClient, ServiceExt,
};
use std::time::Duration;

use crate::config::StreamableHttpConfig;

/// Create an MCP client using streamable HTTP transport
pub async fn create_http_client(
    config: &StreamableHttpConfig,
) -> Result<RunningService<RoleClient, ()>> {
    // Extract Authorization header value from config if present
    let auth_header = config.headers.get("Authorization");
    // build the config to use with this transport
    let mut http_config = StreamableHttpClientTransportConfig::with_uri(config.url.clone());
    // set auth header if present
    if let Some(auth_header) = auth_header {
        http_config = http_config.auth_header(auth_header.resolve()?);
    }
    // Create streamable HTTP transport using the reqwest client
    let transport = StreamableHttpClientTransport::from_config(http_config);

    // Create and initialize the client with timeout
    let service = tokio::time::timeout(
        Duration::from_secs(config.timeout_seconds),
        ().serve(transport),
    )
    .await
    .map_err(|_| anyhow!("Timeout during MCP service initialization"))?
    .map_err(|e| anyhow!("Failed to initialize MCP service: {}", e))?;

    Ok(service)
}
