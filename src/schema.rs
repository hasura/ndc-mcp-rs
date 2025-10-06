use ndc_sdk::models::{
    self, ArgumentInfo, ArgumentName, CollectionInfo, FunctionInfo, ObjectField, ObjectType,
    ProcedureInfo, Type,
};
use rmcp::model::{Resource, Tool};
use schemars::schema::ObjectValidation;
use std::collections::{BTreeMap, HashMap};

use crate::config::{ConnectorConfig, McpServerName};

/// Check if a tool is read-only based on annotations
fn is_read_only_tool(tool: &Tool) -> bool {
    // For now, we'll use a simple heuristic: if the tool name starts with "get" or "list",
    // we'll consider it read-only
    let name = tool.name.to_string().to_lowercase();
    name.starts_with("get")
        || name.starts_with("list")
        || name.starts_with("find")
        || name.starts_with("search")
}

fn tool_arguments_schema(
    input_schema: &rmcp::model::JsonObject,
) -> BTreeMap<ArgumentName, ArgumentInfo> {
    // Parse input schema as ObjectValidation
    let input_schema: ObjectValidation =
        serde_json::from_value(serde_json::Value::Object(input_schema.clone())).unwrap();
    let mut arguments = BTreeMap::new();
    // Iterate over properties
    for (property_name, property) in input_schema.properties {
        // Build argument name
        let argument_name = ArgumentName::new(property_name.as_str().into());
        // Use "String" type for all arguments
        let mut argument_type = Type::Named {
            name: "String".to_string().into(),
        };
        if !input_schema.required.contains(&property_name) {
            argument_type = Type::Nullable {
                underlying_type: Box::new(argument_type),
            };
        }
        let argument_info = ArgumentInfo {
            description: property
                .into_object()
                .metadata
                .and_then(|m| m.description.clone()),
            argument_type,
        };
        // Insert argument info into arguments
        arguments.insert(argument_name, argument_info.clone());
    }
    arguments
}

/// Map MCP resources to NDC collections
fn map_resources_to_collections(
    server_name: &McpServerName,
    resources: &HashMap<String, Resource>,
) -> Vec<CollectionInfo> {
    let mut collections = Vec::new();

    for (resource_id, resource) in resources {
        // Create collection info with server_name prefix
        let description = resource.description.clone().map(|d| d.to_string());
        let collection = CollectionInfo {
            name: format!("{}__{}", server_name.0, resource_id).into(),
            description,
            arguments: BTreeMap::new(), // No arguments for collections
            collection_type: "ResourceOutput".to_string().into(),
            uniqueness_constraints: BTreeMap::new(),
            relational_mutations: None,
        };

        collections.push(collection);
    }

    collections
}

/// Map read-only MCP tools to NDC functions
fn map_tools_to_functions(
    server_name: &McpServerName,
    tools: &HashMap<String, Tool>,
) -> Vec<FunctionInfo> {
    let mut functions = Vec::new();

    for (tool_id, tool) in tools {
        // Check if tool is read-only based on annotations
        if is_read_only_tool(tool) {
            // Convert arguments to BTreeMap with ArgumentInfo
            let arguments = tool_arguments_schema(&tool.input_schema);

            // Create function info with server_name prefix
            let function = FunctionInfo {
                name: format!("{}__{}", server_name.0, tool_id).into(),
                description: tool.description.as_ref().map(|d| d.to_string()),
                arguments,
                result_type: Type::Named {
                    name: "ToolOutput".to_string().into(),
                },
            };

            functions.push(function);
        }
    }

    functions
}

/// Map mutable MCP tools to NDC procedures
fn map_tools_to_procedures(
    server_name: &McpServerName,
    tools: &HashMap<String, Tool>,
) -> Vec<ProcedureInfo> {
    let mut procedures = Vec::new();

    for (tool_id, tool) in tools {
        // Check if tool is mutable (not read-only) based on annotations
        if !is_read_only_tool(tool) {
            // Convert arguments to BTreeMap with ArgumentInfo
            let arguments = tool_arguments_schema(&tool.input_schema);

            // Create procedure info with server_name prefix
            let procedure = ProcedureInfo {
                name: format!("{}__{}", server_name.0, tool_id).into(),
                description: tool.description.as_ref().map(|d| d.to_string()),
                arguments,
                result_type: Type::Named {
                    name: "ToolOutput".to_string().into(),
                },
            };

            procedures.push(procedure);
        }
    }

    procedures
}

/// Create object types for resources and tools
fn create_object_types() -> BTreeMap<String, ObjectType> {
    let mut object_types = BTreeMap::new();

    // Create ResourceOutput type
    let mut resource_fields = BTreeMap::new();
    resource_fields.insert(
        "content".into(),
        ObjectField {
            description: Some("The content of the resource".to_string()),
            r#type: Type::Named {
                name: "String".to_string().into(),
            },
            arguments: BTreeMap::new(),
        },
    );

    object_types.insert(
        "ResourceOutput".to_string(),
        ObjectType {
            description: Some("Output type for MCP resources".to_string()),
            fields: resource_fields,
            foreign_keys: BTreeMap::new(),
        },
    );

    // Create Content Object
    let mut content_fields = BTreeMap::new();
    content_fields.insert(
        "type".into(),
        ObjectField {
            description: Some("The type of the content".to_string()),
            r#type: Type::Named {
                name: "String".to_string().into(),
            },
            arguments: BTreeMap::new(),
        },
    );
    content_fields.insert(
        "text".into(),
        ObjectField {
            description: Some("The value of the content".to_string()),
            r#type: Type::Named {
                name: "String".to_string().into(),
            },
            arguments: BTreeMap::new(),
        },
    );
    object_types.insert(
        "Content".to_string(),
        ObjectType {
            description: Some("Content type for MCP tools".to_string()),
            fields: content_fields,
            foreign_keys: BTreeMap::new(),
        },
    );

    // Create ToolOutput type
    let mut tool_fields = BTreeMap::new();
    tool_fields.insert(
        "content".into(),
        ObjectField {
            description: Some("The output of the tool".to_string()),
            r#type: Type::Array {
                element_type: Box::new(Type::Named {
                    name: "Content".to_string().into(),
                }),
            },
            arguments: BTreeMap::new(),
        },
    );

    object_types.insert(
        "ToolOutput".to_string(),
        ObjectType {
            description: Some("Output type for MCP tools".to_string()),
            fields: tool_fields,
            foreign_keys: BTreeMap::new(),
        },
    );

    object_types
}

fn create_scalar_types() -> BTreeMap<models::ScalarTypeName, models::ScalarType> {
    let mut scalar_types = BTreeMap::new();

    // Add String scalar type
    scalar_types.insert(
        "String".to_string().into(),
        models::ScalarType {
            representation: models::TypeRepresentation::String,
            aggregate_functions: BTreeMap::new(),
            comparison_operators: BTreeMap::new(),
            extraction_functions: BTreeMap::new(),
        },
    );

    // Add Json scalar type
    scalar_types.insert(
        "Json".to_string().into(),
        models::ScalarType {
            representation: models::TypeRepresentation::JSON,
            aggregate_functions: BTreeMap::new(),
            comparison_operators: BTreeMap::new(),
            extraction_functions: BTreeMap::new(),
        },
    );

    scalar_types
}

/// Generate the NDC schema from the connector state
pub fn generate_schema(config: &ConnectorConfig) -> models::SchemaResponse {
    let mut collections = Vec::new();
    let mut functions = Vec::new();
    let mut procedures = Vec::new();

    // Process each MCP client
    for (server_name, client) in &config.servers {
        // Add error handling for empty tools/resources
        if client.tools.is_empty() && client.resources.is_empty() {
            tracing::warn!("MCP server {} has no tools or resources", server_name.0);
            continue;
        }

        // Map resources to collections
        let server_collections = map_resources_to_collections(server_name, &client.resources);
        collections.extend(server_collections);

        // Map read-only tools to functions
        let server_functions = map_tools_to_functions(server_name, &client.tools);
        functions.extend(server_functions);

        // Map mutable tools to procedures
        let server_procedures = map_tools_to_procedures(server_name, &client.tools);
        procedures.extend(server_procedures);
    }

    // Create object types
    let object_types = create_object_types();

    // Create scalar types
    let scalar_types = create_scalar_types();

    // Convert object types to use ObjectTypeName keys
    let mut typed_object_types = BTreeMap::new();
    for (name, obj_type) in object_types {
        typed_object_types.insert(name.into(), obj_type);
    }

    // Create schema response
    models::SchemaResponse {
        collections,
        functions,
        procedures,
        object_types: typed_object_types,
        scalar_types,
        capabilities: None,
        request_arguments: None,
    }
}
