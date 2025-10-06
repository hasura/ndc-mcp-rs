use rmcp::{model::{ErrorCode, ErrorData}, ServiceError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use std::fs;

use super::transport::create_mcp_client;

pub static CONFIG_FILE_NAME: &str = "configuration.json";
pub static SERVERS_FILE_NAME: &str = "servers.yaml";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub config: McpServerConfig,
    pub resources: HashMap<String, rmcp::model::Resource>,
    pub tools: HashMap<String, rmcp::model::Tool>,
}

/// Configuration for the NDC MCP connector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    /// List of MCP servers
    pub servers: HashMap<McpServerName, McpServer>,
}

impl ConnectorConfig {
    /// Load configuration from a file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ConnectorConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Servers {
    pub servers: HashMap<McpServerName, McpServerConfig>,
}

pub async fn generate_config(servers: Servers) -> Result<ConnectorConfig> {
    let mut server_configs: HashMap<McpServerName, McpServer> = HashMap::new();
    for (server_name, server_config) in servers.servers {
        // Create MCP client
        let service = create_mcp_client(&server_config).await.map_err(|e| {
            anyhow!("Failed to create MCP client for server {}: {}", server_name.0, e)
        })?;

        // List resources (only if server supports resources)
        let mut resources = HashMap::new();

        match service.list_all_resources().await {
            Ok(resources_result) => {
                for resource in resources_result {
                    resources.insert(resource.raw.name.clone(), resource);
                }
            },
            Err(err) => {
                let err_message = format!("Failed to list resources for server {}: {}", server_name.0, err);
                if !is_method_not_found_error(&err) {
                    return Err(anyhow!(err_message));
                }
            }
        }

        // List tools (only if server supports tools)
        let mut tools = HashMap::new();
        match service.list_all_tools().await {
            Ok(tools_result) => {
                for tool in tools_result {
                    tools.insert(tool.name.to_string(), tool);
                }
            },
            Err(err) => {
                let err_message = format!("Failed to list tools for server {}: {}", server_name.0, err);
                if !is_method_not_found_error(&err) {
                    return Err(anyhow!(err_message));
                }
            }
        }

        // Add server config to the list
        server_configs.insert(server_name.clone(), McpServer {
            config: server_config,
            resources,
            tools,
        });
    }
    Ok(ConnectorConfig { servers: server_configs })
}

fn is_method_not_found_error(err: &ServiceError) -> bool {
    matches!(err, ServiceError::McpError(ErrorData { code: ErrorCode::METHOD_NOT_FOUND, .. }))
}
