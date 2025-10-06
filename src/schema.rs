use ndc_sdk::models::{
    self, ArgumentInfo, ArgumentName, CollectionInfo, FunctionInfo, ObjectField, ObjectType,
    ProcedureInfo, Type,
};
use rmcp::model::{Resource, Tool};
use schemars::schema::{InstanceType, ObjectValidation, Schema, SingleOrVec};
use std::collections::{BTreeMap, HashMap};

use crate::config::McpServerName;
use crate::state::ConnectorState;

/// Check if a tool is read-only based on annotations
fn is_read_only_tool(tool: &Tool) -> bool {
    // For now, we'll use a simple heuristic: if the tool name starts with "get" or "list",
    // we'll consider it read-only
    let name = tool.name.to_string().to_lowercase();
    name.starts_with("get")
        || name.starts_with("list")
        || name.starts_with("find")
        || name.starts_with("search")
        // Check if the tool has annotations and if the read_only_hint is true
        || tool
            .annotations
            .as_ref()
            .and_then(|annotations| annotations.read_only_hint)
            .unwrap_or(false)
}

/// Create a named type with the given type name
fn create_named_type(type_name: &str) -> Type {
    Type::Named {
        name: type_name.to_string().into(),
    }
}

/// Map a single instance type to NDC type
fn map_instance_type_to_ndc(instance_type: &InstanceType) -> Type {
    match instance_type {
        InstanceType::String => create_named_type("String"),
        InstanceType::Number => create_named_type("Float"),
        InstanceType::Integer => create_named_type("Int"),
        InstanceType::Boolean => create_named_type("Boolean"),
        _ => create_named_type("String"), // Fallback to String for Object, Null, etc.
    }
}

/// Handle array type mapping by examining items schema
fn map_array_type(schema_obj: &schemars::schema::SchemaObject) -> Type {
    if let Some(items) = &schema_obj.array {
        if let Some(items_schema) = &items.items {
            match items_schema {
                schemars::schema::SingleOrVec::Single(item_schema) => {
                    let element_type = map_schema_to_ndc_type(item_schema);
                    Type::Array {
                        element_type: Box::new(element_type),
                    }
                }
                schemars::schema::SingleOrVec::Vec(item_schemas) => {
                    // For multiple item schemas, use first one or fallback to String
                    if item_schemas.len() == 1 {
                        let element_type = map_schema_to_ndc_type(&item_schemas[0]);
                        Type::Array {
                            element_type: Box::new(element_type),
                        }
                    } else {
                        Type::Array {
                            element_type: Box::new(create_named_type("String")),
                        }
                    }
                }
            }
        } else {
            // No items schema specified, use String array
            Type::Array {
                element_type: Box::new(create_named_type("String")),
            }
        }
    } else {
        // No array validation specified, use String array
        Type::Array {
            element_type: Box::new(create_named_type("String")),
        }
    }
}

/// Map JSON schema type to NDC type
fn map_schema_to_ndc_type(schema: &Schema) -> Type {
    match schema {
        Schema::Bool(_) => create_named_type("String"), // Fallback to String
        Schema::Object(schema_obj) => {
            if let Some(instance_type) = &schema_obj.instance_type {
                match instance_type {
                    SingleOrVec::Single(instance_type) => match instance_type.as_ref() {
                        InstanceType::Array => map_array_type(schema_obj),
                        other => map_instance_type_to_ndc(other),
                    },
                    SingleOrVec::Vec(types) => {
                        // For multiple types, use first one or fallback to String
                        if types.len() == 1 {
                            match &types[0] {
                                InstanceType::Array => map_array_type(schema_obj),
                                other => map_instance_type_to_ndc(other),
                            }
                        } else {
                            create_named_type("String")
                        }
                    }
                }
            } else {
                // No instance type specified, default to String
                create_named_type("String")
            }
        }
    }
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
        // Map JSON schema type to NDC type
        let mut argument_type = map_schema_to_ndc_type(&property);
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
    // content field
    tool_fields.insert(
        "content".into(),
        ObjectField {
            description: Some("The text output of the tool".to_string()),
            r#type: Type::Array {
                element_type: Box::new(Type::Named {
                    name: "Content".to_string().into(),
                }),
            },
            arguments: BTreeMap::new(),
        },
    );

    // optional structured content field
    tool_fields.insert(
        "structured_content".into(),
        ObjectField {
            description: Some(
                "The structured output of the tool. This is a JSON string.".to_string(),
            ),
            r#type: Type::Nullable {
                underlying_type: Box::new(Type::Named {
                    name: "String".to_string().into(),
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

/// Create a scalar type with the given representation
fn create_scalar_type(representation: models::TypeRepresentation) -> models::ScalarType {
    models::ScalarType {
        representation,
        aggregate_functions: BTreeMap::new(),
        comparison_operators: BTreeMap::new(),
        extraction_functions: BTreeMap::new(),
    }
}

fn create_scalar_types() -> BTreeMap<models::ScalarTypeName, models::ScalarType> {
    let mut scalar_types = BTreeMap::new();

    // Add core scalar types
    scalar_types.insert(
        "String".to_string().into(),
        create_scalar_type(models::TypeRepresentation::String),
    );
    scalar_types.insert(
        "Boolean".to_string().into(),
        create_scalar_type(models::TypeRepresentation::Boolean),
    );
    scalar_types.insert(
        "Int".to_string().into(),
        create_scalar_type(models::TypeRepresentation::Int32),
    );
    scalar_types.insert(
        "Float".to_string().into(),
        create_scalar_type(models::TypeRepresentation::Float64),
    );

    scalar_types
}

/// Generate the NDC schema from the connector state
pub fn generate_schema_from_state(state: &ConnectorState) -> models::SchemaResponse {
    let mut collections = Vec::new();
    let mut functions = Vec::new();
    let mut procedures = Vec::new();

    // Process each MCP server from state
    for (server_name, client) in &state.clients {
        // Map resources to collections
        collections.extend(map_resources_to_collections(server_name, &client.resources));

        // Map tools to functions and procedures
        functions.extend(map_tools_to_functions(server_name, &client.tools));
        procedures.extend(map_tools_to_procedures(server_name, &client.tools));
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_map_schema_to_ndc_type_primitives() {
        // Test string type
        let string_schema = serde_json::from_value(json!({
            "type": "string"
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&string_schema);
        match ndc_type {
            Type::Named { name } => assert_eq!(name.as_str(), "String"),
            _ => panic!("Expected Named type"),
        }

        // Test integer type
        let int_schema = serde_json::from_value(json!({
            "type": "integer"
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&int_schema);
        match ndc_type {
            Type::Named { name } => assert_eq!(name.as_str(), "Int"),
            _ => panic!("Expected Named type"),
        }

        // Test number type
        let number_schema = serde_json::from_value(json!({
            "type": "number"
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&number_schema);
        match ndc_type {
            Type::Named { name } => assert_eq!(name.as_str(), "Float"),
            _ => panic!("Expected Named type"),
        }

        // Test boolean type
        let bool_schema = serde_json::from_value(json!({
            "type": "boolean"
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&bool_schema);
        match ndc_type {
            Type::Named { name } => assert_eq!(name.as_str(), "Boolean"),
            _ => panic!("Expected Named type"),
        }
    }

    #[test]
    fn test_map_schema_to_ndc_type_arrays() {
        // Test array of strings
        let string_array_schema = serde_json::from_value(json!({
            "type": "array",
            "items": {
                "type": "string"
            }
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&string_array_schema);
        match ndc_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Named { name } => assert_eq!(name.as_str(), "String"),
                _ => panic!("Expected Named element type"),
            },
            _ => panic!("Expected Array type"),
        }

        // Test array of integers
        let int_array_schema = serde_json::from_value(json!({
            "type": "array",
            "items": {
                "type": "integer"
            }
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&int_array_schema);
        match ndc_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Named { name } => assert_eq!(name.as_str(), "Int"),
                _ => panic!("Expected Named element type"),
            },
            _ => panic!("Expected Array type"),
        }

        // Test array of numbers
        let number_array_schema = serde_json::from_value(json!({
            "type": "array",
            "items": {
                "type": "number"
            }
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&number_array_schema);
        match ndc_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Named { name } => assert_eq!(name.as_str(), "Float"),
                _ => panic!("Expected Named element type"),
            },
            _ => panic!("Expected Array type"),
        }

        // Test array of booleans
        let bool_array_schema = serde_json::from_value(json!({
            "type": "array",
            "items": {
                "type": "boolean"
            }
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&bool_array_schema);
        match ndc_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Named { name } => assert_eq!(name.as_str(), "Boolean"),
                _ => panic!("Expected Named element type"),
            },
            _ => panic!("Expected Array type"),
        }

        // Test array without items schema (should default to String array)
        let generic_array_schema = serde_json::from_value(json!({
            "type": "array"
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&generic_array_schema);
        match ndc_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Named { name } => assert_eq!(name.as_str(), "String"),
                _ => panic!("Expected Named element type"),
            },
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_map_schema_to_ndc_type_nested_arrays() {
        // Test array of arrays of strings
        let nested_array_schema = serde_json::from_value(json!({
            "type": "array",
            "items": {
                "type": "array",
                "items": {
                    "type": "string"
                }
            }
        }))
        .unwrap();
        let ndc_type = map_schema_to_ndc_type(&nested_array_schema);
        match ndc_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Array {
                    element_type: inner_element_type,
                } => match inner_element_type.as_ref() {
                    Type::Named { name } => assert_eq!(name.as_str(), "String"),
                    _ => panic!("Expected Named inner element type"),
                },
                _ => panic!("Expected Array element type"),
            },
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_tool_arguments_schema_with_arrays() {
        // Test a realistic schema with various array types
        let input_schema = json!({
            "type": "object",
            "properties": {
                "names": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Array of names"
                },
                "scores": {
                    "type": "array",
                    "items": {
                        "type": "number"
                    },
                    "description": "Array of scores"
                },
                "flags": {
                    "type": "array",
                    "items": {
                        "type": "boolean"
                    },
                    "description": "Array of boolean flags"
                },
                "ids": {
                    "type": "array",
                    "items": {
                        "type": "integer"
                    },
                    "description": "Array of integer IDs"
                },
                "mixed_data": {
                    "type": "array",
                    "description": "Array with no specific item type"
                }
            },
            "required": ["names", "scores"]
        });

        let input_schema_obj = input_schema.as_object().unwrap().clone();
        let arguments = tool_arguments_schema(&input_schema_obj);

        // Check that we have the expected arguments
        assert_eq!(arguments.len(), 5);

        // Check names argument (required string array)
        let names_arg = arguments.get(&ArgumentName::new("names".into())).unwrap();
        match &names_arg.argument_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Named { name } => assert_eq!(name.as_str(), "String"),
                _ => panic!("Expected String element type for names"),
            },
            _ => panic!("Expected Array type for names"),
        }

        // Check scores argument (required number array)
        let scores_arg = arguments.get(&ArgumentName::new("scores".into())).unwrap();
        match &scores_arg.argument_type {
            Type::Array { element_type } => match element_type.as_ref() {
                Type::Named { name } => assert_eq!(name.as_str(), "Float"),
                _ => panic!("Expected Float element type for scores"),
            },
            _ => panic!("Expected Array type for scores"),
        }

        // Check flags argument (optional boolean array)
        let flags_arg = arguments.get(&ArgumentName::new("flags".into())).unwrap();
        match &flags_arg.argument_type {
            Type::Nullable { underlying_type } => match underlying_type.as_ref() {
                Type::Array { element_type } => match element_type.as_ref() {
                    Type::Named { name } => assert_eq!(name.as_str(), "Boolean"),
                    _ => panic!("Expected Boolean element type for flags"),
                },
                _ => panic!("Expected Array underlying type for flags"),
            },
            _ => panic!("Expected Nullable type for flags"),
        }

        // Check ids argument (optional integer array)
        let ids_arg = arguments.get(&ArgumentName::new("ids".into())).unwrap();
        match &ids_arg.argument_type {
            Type::Nullable { underlying_type } => match underlying_type.as_ref() {
                Type::Array { element_type } => match element_type.as_ref() {
                    Type::Named { name } => assert_eq!(name.as_str(), "Int"),
                    _ => panic!("Expected Int element type for ids"),
                },
                _ => panic!("Expected Array underlying type for ids"),
            },
            _ => panic!("Expected Nullable type for ids"),
        }

        // Check mixed_data argument (optional String array)
        let mixed_arg = arguments
            .get(&ArgumentName::new("mixed_data".into()))
            .unwrap();
        match &mixed_arg.argument_type {
            Type::Nullable { underlying_type } => match underlying_type.as_ref() {
                Type::Array { element_type } => match element_type.as_ref() {
                    Type::Named { name } => assert_eq!(name.as_str(), "String"),
                    _ => panic!("Expected String element type for mixed_data"),
                },
                _ => panic!("Expected Array underlying type for mixed_data"),
            },
            _ => panic!("Expected Nullable type for mixed_data"),
        }
    }
}
