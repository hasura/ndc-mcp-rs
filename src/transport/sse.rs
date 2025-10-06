use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rmcp::{service::RunningService, RoleClient};
use url::Url;

use crate::config::SseConfig;

/// Create an MCP client using SSE transport
pub async fn create_sse_client(config: &SseConfig) -> Result<RunningService<RoleClient, ()>> {
    // Extract fields from the config
    // Parse URL
    let _url = Url::parse(&config.url)?;

    // Create HTTP client
    let _client = reqwest::Client::new();

    // Create headers
    let mut _headers = HeaderMap::new();

    // Add headers
    for (key, value) in &config.headers {
        let header_name = HeaderName::from_bytes(key.as_bytes())?;
        let header_value = HeaderValue::from_str(value)?;
        _headers.insert(header_name, header_value);
    }

    // SSE transport is deprecated in favor of streamable HTTP transport
    Err(anyhow!("SSE transport is deprecated. Please use 'http' transport type instead for streamable HTTP transport."))
}
