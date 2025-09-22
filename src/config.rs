use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use std::fs;

/// Configuration for a stdio-based MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioConfig {
    /// Command to start the server executable
    pub command: String,

    /// Arguments passed to the command
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables for the server
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Path to an .env file from which to load additional environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
}

/// Configuration for an SSE-based MCP server (DEPRECATED - use HTTP instead)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfig {
    /// URL of the server
    pub url: String,

    /// HTTP headers for the server
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Configuration for a streamable HTTP-based MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamableHttpConfig {
    /// URL of the server
    pub url: String,

    /// HTTP headers for the server
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Timeout for HTTP requests in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_timeout() -> u64 {
    30
}

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio(StdioConfig),
    #[serde(rename = "sse")]
    Sse(SseConfig),
    #[serde(rename = "http")]
    Http(StreamableHttpConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct McpServerName(pub String);

/// Configuration for the NDC MCP connector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    /// List of MCP servers
    pub servers: HashMap<McpServerName, McpServerConfig>,
}

impl ConnectorConfig {
    /// Load configuration from a file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ConnectorConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}
