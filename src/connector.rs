//! This defines a `Connector` implementation for MCP (Model Context Protocol).
//! The routes are defined here.

use async_trait::async_trait;
use http::StatusCode;
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use ndc_sdk::connector::ErrorResponse;
use ndc_sdk::connector::{Connector, ConnectorSetup};
use ndc_sdk::json_response::JsonResponse;
use ndc_sdk::models;
use rmcp::{
    model::{ErrorCode, ErrorData},
    ServiceError,
};

use crate::config::{
    ConnectorConfig, EnvVariableValue, McpServerConfig, McpServerName, StdioConfig,
    StreamableHttpConfig,
};
use crate::schema::generate_schema_from_state;
use crate::state::{ConnectorState, McpClient};
use crate::transport::create_mcp_client;

/// NDC MCP Connector
#[derive(Default)]
pub struct McpConnector;

/// Setup for the MCP connector
#[derive(Default)]
pub struct McpConnectorSetup;

/// Helper function to initialize MCP clients and build schema
async fn initialize_mcp_clients(
    configuration: &ConnectorConfig,
) -> Result<ConnectorState, ErrorResponse> {
    let mut connector_state = ConnectorState::new();
    // Initialize clients
    for (server_name, server_config) in &configuration.servers {
        // Create MCP client
        let service = create_mcp_client(server_config).await.map_err(|e| {
            ErrorResponse::new(
                StatusCode::BAD_REQUEST,
                format!("Failed to create MCP client: {}", e),
                serde_json::Value::Null,
            )
        })?;

        // Introspect resources from the server
        let mut resources = HashMap::new();
        match service.list_all_resources().await {
            Ok(resources_result) => {
                for resource in resources_result {
                    resources.insert(resource.raw.name.clone(), resource);
                }
            }
            Err(err) => {
                let err_message = format!(
                    "Failed to list resources for server {}: {}",
                    server_name.0, err
                );
                if !is_method_not_found_error(&err) {
                    return Err(ErrorResponse::new(
                        StatusCode::BAD_REQUEST,
                        err_message,
                        serde_json::Value::Null,
                    ));
                }
                // If method not found, just log and continue with empty resources
                tracing::info!("Server {} does not support resources", server_name.0);
            }
        }

        // Introspect tools from the server
        let mut tools = HashMap::new();
        match service.list_all_tools().await {
            Ok(tools_result) => {
                for tool in tools_result {
                    tools.insert(tool.name.to_string(), tool);
                }
            }
            Err(err) => {
                let err_message =
                    format!("Failed to list tools for server {}: {}", server_name.0, err);
                if !is_method_not_found_error(&err) {
                    return Err(ErrorResponse::new(
                        StatusCode::BAD_REQUEST,
                        err_message,
                        serde_json::Value::Null,
                    ));
                }
                // If method not found, just log and continue with empty tools
                tracing::info!("Server {} does not support tools", server_name.0);
            }
        }

        // Create client with introspected data
        let client = McpClient {
            service,
            resources,
            tools,
        };

        // Add client to state
        connector_state.add_client(server_name.clone(), client);
    }

    Ok(connector_state)
}

/// Helper function to check if an error is a method not found error
fn is_method_not_found_error(err: &ServiceError) -> bool {
    matches!(
        err,
        ServiceError::McpError(ErrorData {
            code: ErrorCode::METHOD_NOT_FOUND,
            ..
        })
    )
}

#[async_trait]
impl Connector for McpConnector {
    type Configuration = ConnectorConfig;
    type State = Arc<ConnectorState>;

    fn connector_name() -> &'static str {
        "mcp-connector"
    }

    fn connector_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn fetch_metrics(
        _configuration: &Self::Configuration,
        _state: &Self::State,
    ) -> Result<(), ErrorResponse> {
        Ok(())
    }

    async fn get_health_readiness(
        _configuration: &Self::Configuration,
        _state: &Self::State,
    ) -> Result<(), ErrorResponse> {
        Ok(())
    }

    async fn get_capabilities() -> models::Capabilities {
        models::Capabilities {
            relationships: None,
            query: models::QueryCapabilities {
                variables: None,
                aggregates: None,
                explain: None,
                nested_fields: models::NestedFieldCapabilities {
                    filter_by: None,
                    order_by: None,
                    aggregates: None,
                    nested_collections: None,
                },
                exists: models::ExistsCapabilities {
                    nested_collections: None,
                    unrelated: None,
                    named_scopes: None,
                    nested_scalar_collections: None,
                },
            },
            mutation: models::MutationCapabilities {
                transactional: None,
                explain: None,
            },
            relational_query: None,
            relational_mutation: None,
        }
    }

    async fn get_schema(
        configuration: &Self::Configuration,
    ) -> Result<JsonResponse<models::SchemaResponse>, ErrorResponse> {
        // Initialize temporary state to introspect MCP servers and generate schema
        let state = initialize_mcp_clients(configuration).await?;

        // Generate schema from the introspected state
        let schema = generate_schema_from_state(&state);
        Ok(schema.into())
    }

    async fn query_explain(
        _configuration: &Self::Configuration,
        _state: &Self::State,
        _request: models::QueryRequest,
    ) -> Result<JsonResponse<models::ExplainResponse>, ErrorResponse> {
        Err(ErrorResponse::new(
            StatusCode::NOT_IMPLEMENTED,
            "Explain not supported".to_string(),
            serde_json::Value::Null,
        ))
    }

    async fn mutation_explain(
        _configuration: &Self::Configuration,
        _state: &Self::State,
        _request: models::MutationRequest,
    ) -> Result<JsonResponse<models::ExplainResponse>, ErrorResponse> {
        Err(ErrorResponse::new(
            StatusCode::NOT_IMPLEMENTED,
            "Explain not supported".to_string(),
            serde_json::Value::Null,
        ))
    }

    async fn query(
        _configuration: &Self::Configuration,
        state: &Self::State,
        request: models::QueryRequest,
    ) -> Result<JsonResponse<models::QueryResponse>, ErrorResponse> {
        // Parse the collection or function name to extract server_name and resource/tool name
        let name = request.collection.to_string();
        let parts: Vec<&str> = name.split("__").collect();

        if parts.len() != 2 {
            return Err(ErrorResponse::new(
                StatusCode::BAD_REQUEST,
                format!("Invalid collection or function name: {}", name),
                serde_json::Value::Null,
            ));
        }

        let server_name = parts[0];
        let resource_or_tool_name = parts[1];

        // Find the client for this server
        let client = state
            .clients
            .get(&McpServerName(server_name.to_string()))
            .ok_or_else(|| {
                ErrorResponse::new(
                    StatusCode::NOT_FOUND,
                    format!("Server not found: {}", server_name),
                    serde_json::Value::Null,
                )
            })?;

        // Check if this is a resource (collection) or a tool (function)
        if let Some(resource) = client.resources.get(resource_or_tool_name) {
            // This is a resource (collection)
            // Read the resource
            let read_request = rmcp::model::ReadResourceRequestParam {
                uri: resource.raw.uri.clone(),
            };

            let result = client
                .service
                .read_resource(read_request)
                .await
                .map_err(|e| {
                    ErrorResponse::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read resource: {}", e),
                        serde_json::Value::Null,
                    )
                })?;

            // Convert content to a row
            let content = serde_json::to_value(&result.contents).unwrap_or(Value::Null);

            // Create a simple response with one row
            let mut row = IndexMap::new();
            row.insert(
                models::FieldName::new("content".into()),
                models::RowFieldValue(content),
            );
            let rowset = models::RowSet {
                rows: Some(vec![row]),
                aggregates: None,
                groups: None,
            };

            // Return response with a single row
            Ok(models::QueryResponse(vec![rowset]).into())
        } else if let Some(tool) = client.tools.get(resource_or_tool_name) {
            // Extract input from arguments if provided
            let mut arguments_map = serde_json::Map::new();
            for (argument_name, argument) in request.arguments {
                if let models::Argument::Literal { value } = argument {
                    arguments_map.insert(argument_name.to_string(), value);
                }
            }

            // Execute the tool
            let call_request = rmcp::model::CallToolRequestParam {
                name: tool.name.clone(),
                arguments: if arguments_map.is_empty() {
                    None
                } else {
                    Some(arguments_map)
                },
            };

            let result = client.service.call_tool(call_request).await.map_err(|e| {
                ErrorResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to execute tool: {}", e),
                    serde_json::Value::Null,
                )
            })?;

            let contents = result
                .content
                .into_iter()
                .filter_map(|content| {
                    if matches!(content.raw, rmcp::model::RawContent::Text { .. }) {
                        Some(content.raw)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            // Convert content to a row
            let mut row = IndexMap::new();
            row.insert(
                "__value".into(),
                models::RowFieldValue(serde_json::json!({"content": contents})),
            );
            let rowset = models::RowSet {
                rows: Some(vec![row]),
                aggregates: None,
                groups: None,
            };

            // Return response with a single row
            Ok(models::QueryResponse(vec![rowset]).into())
        } else {
            Err(ErrorResponse::new(
                StatusCode::NOT_FOUND,
                format!("Resource or tool not found: {}", resource_or_tool_name),
                serde_json::Value::Null,
            ))
        }
    }

    async fn mutation(
        _configuration: &Self::Configuration,
        state: &Self::State,
        request: models::MutationRequest,
    ) -> Result<JsonResponse<models::MutationResponse>, ErrorResponse> {
        // Process each mutation operation
        let mut operation_results = Vec::new();

        for operation in request.operations {
            match operation {
                models::MutationOperation::Procedure {
                    name,
                    arguments,
                    fields: _,
                } => {
                    // Parse the procedure name to extract server_name and tool name
                    let name_str = name.to_string();
                    let parts: Vec<&str> = name_str.split("__").collect();

                    if parts.len() != 2 {
                        return Err(ErrorResponse::new(
                            StatusCode::BAD_REQUEST,
                            format!("Invalid procedure name: {}", name_str),
                            serde_json::Value::Null,
                        ));
                    }

                    let server_name = parts[0];
                    let tool_name = parts[1];

                    // Find the client for this server
                    let client = state
                        .clients
                        .get(&McpServerName(server_name.to_string()))
                        .ok_or_else(|| {
                            ErrorResponse::new(
                                StatusCode::NOT_FOUND,
                                format!("Server not found: {}", server_name),
                                serde_json::Value::Null,
                            )
                        })?;

                    // Check if the tool exists
                    if !client.tools.contains_key(tool_name) {
                        return Err(ErrorResponse::new(
                            StatusCode::NOT_FOUND,
                            format!("Tool not found: {}", tool_name),
                            serde_json::Value::Null,
                        ));
                    }

                    // Extract input from arguments if provided
                    let mut arguments_map = serde_json::Map::new();
                    for (argument_name, value) in arguments {
                        arguments_map.insert(argument_name.to_string(), value);
                    }

                    // Execute the tool
                    let call_request = rmcp::model::CallToolRequestParam {
                        name: tool_name.to_string().into(),
                        arguments: if arguments_map.is_empty() {
                            None
                        } else {
                            Some(arguments_map)
                        },
                    };

                    let result = client.service.call_tool(call_request).await.map_err(|e| {
                        ErrorResponse::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to execute tool: {}", e),
                            serde_json::Value::Null,
                        )
                    })?;

                    let raw_contents = result
                        .content
                        .into_iter()
                        .map(|content| content.raw)
                        .collect::<Vec<_>>();
                    let content = serde_json::to_value(&raw_contents).unwrap_or(Value::Null);

                    // Convert the result to a JSON value
                    operation_results.push(models::MutationOperationResults::Procedure {
                        result: serde_json::json!({"content": content}),
                    });
                }
            }
        }

        Ok(models::MutationResponse { operation_results }.into())
    }
}

#[async_trait]
impl ConnectorSetup for McpConnectorSetup {
    type Connector = McpConnector;

    async fn parse_configuration(
        &self,
        configuration_dir: &Path,
    ) -> Result<<Self::Connector as Connector>::Configuration, ErrorResponse> {
        // Load configuration from file
        let config_path = configuration_dir.join("configuration.json");
        let config = ConnectorConfig::from_file(&config_path).map_err(|e| {
            ErrorResponse::new(
                StatusCode::BAD_REQUEST,
                format!("Failed to load configuration: {}", e),
                serde_json::Value::Null,
            )
        })?;

        // Let's validate the env variables
        for server_config in config.servers.values() {
            match server_config {
                McpServerConfig::Stdio(StdioConfig { env, .. }) => {
                    validate_env_variables(env)?;
                }
                McpServerConfig::Http(StreamableHttpConfig { headers, .. }) => {
                    validate_env_variables(headers)?;
                }
                _ => {}
            }
        }
        Ok(config)
    }

    async fn try_init_state(
        &self,
        configuration: &<Self::Connector as Connector>::Configuration,
        _metrics: &mut prometheus::Registry,
    ) -> Result<<Self::Connector as Connector>::State, ErrorResponse> {
        // Initialize MCP clients
        let state = initialize_mcp_clients(configuration).await?;
        Ok(Arc::new(state))
    }
}

fn validate_env_variables(env: &HashMap<String, EnvVariableValue>) -> Result<(), ErrorResponse> {
    for (key, value) in env {
        value.resolve().map_err(|e| {
            ErrorResponse::new(
                StatusCode::BAD_REQUEST,
                format!("Failed to resolve environment variable {}: {}", key, e),
                serde_json::Value::Null,
            )
        })?;
    }
    Ok(())
}
