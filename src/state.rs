use std::collections::HashMap;
use rmcp::{model::{Resource, Tool}, service::Peer, RoleClient};

use crate::config::{McpServerConfig, McpServerName};

/// Represents a connected MCP client
pub struct McpClient {
    /// The peer connection to the MCP server
    pub peer: Peer<RoleClient>,
    /// The configuration used to connect to this server
    #[allow(dead_code)]
    pub config: McpServerConfig,
    /// Resources provided by this server
    pub resources: HashMap<String, Resource>,
    /// Tools provided by this server
    pub tools: HashMap<String, Tool>,
}

/// The state of the connector
#[derive(Default)]
pub struct ConnectorState {
    /// Connected MCP clients
    pub clients: HashMap<McpServerName, McpClient>,
}

impl ConnectorState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Add a client to the state
    pub fn add_client(&mut self, name: McpServerName, client: McpClient) {
        // Add the client
        self.clients.insert(name, client);
    }
}
