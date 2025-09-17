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

/// Configuration for an SSE-based MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfig {
    /// URL of the server
    pub url: String,

    /// HTTP headers for the server
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio(StdioConfig),
    #[serde(rename = "sse")]
    Sse(SseConfig),
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
