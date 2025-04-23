use anyhow::{Result, anyhow};
use rmcp::{service::Peer, RoleClient};
use url::Url;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::config::McpServerConfig;

/// Create an MCP client using SSE transport
pub async fn create_sse_client(config: &McpServerConfig) -> Result<Peer<RoleClient>> {
    // Extract fields from the config
    if let McpServerConfig::Sse(sse_config) = config {
        // Parse URL
        let _url = Url::parse(&sse_config.url)?;

        // Create HTTP client
        let _client = reqwest::Client::new();

        // Create headers
        let mut _headers = HeaderMap::new();

        // Add headers
        for (key, value) in &sse_config.headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())?;
            let header_value = HeaderValue::from_str(value)?;
            _headers.insert(header_name, header_value);
        }
    } else {
        return Err(anyhow!("Invalid server configuration type for SSE transport"));
    }

    // For now, return an error since we need to implement SSE transport properly
    Err(anyhow!("SSE transport not yet implemented"))
}
