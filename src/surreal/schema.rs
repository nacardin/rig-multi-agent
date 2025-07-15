use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{SurrealDbConfig, SurrealError, execute_query};

/// Arguments for the SurrealDB schema tool
#[derive(Deserialize, Serialize)]
pub struct SurrealSchemaArgs {
    /// Name of the table to get schema information for
    pub table_name: String,
}

/// Response structure for table schema
#[derive(Serialize, Deserialize, Debug)]
pub struct TableColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<TableColumn>,
}

/// SurrealDB Schema Tool for retrieving table column information
#[derive(Clone)]
pub struct SurrealSchemaTool {
    config: SurrealDbConfig,
}

impl SurrealSchemaTool {
    /// Create a new SurrealDB schema tool with the provided configuration
    pub fn new(config: SurrealDbConfig) -> Self {
        Self { config }
    }

    /// Get structured schema information for a table (for direct use, not AI agents)
    pub async fn get_schema(&self, table_name: &str) -> Result<TableSchema, SurrealError> {
        // Validate table name
        if table_name.trim().is_empty() {
            return Err(SurrealError::InvalidInput(
                "Table name cannot be empty".to_string(),
            ));
        }

        // Construct the INFO TABLE query
        let query = format!("INFO FOR TABLE {table_name}");

        // Execute the query using the shared function
        let result = execute_query(&self.config, &query).await?;

        // Parse the result into structured column information
        let schema = self._parse_table_info(table_name, &result)?;

        Ok(schema)
    }

    /// Parse SurrealDB INFO TABLE response into structured column information
    fn _parse_table_info(
        &self,
        table_name: &str,
        info_result: &Value,
    ) -> Result<TableSchema, SurrealError> {
        let fields = info_result
            .get("fields")
            .ok_or_else(|| SurrealError::QueryError("No fields found in table info".to_string()))?;

        let mut columns = Vec::new();

        if let Value::Object(fields_obj) = fields {
            for (field_name, field_info) in fields_obj {
                let field_type = match field_info.get("type") {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Object(obj)) => {
                        // Handle complex types like arrays, objects, etc.
                        if let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) {
                            match kind {
                                "array" => {
                                    if let Some(items) = obj.get("items").and_then(|v| v.as_str()) {
                                        format!("array<{items}>")
                                    } else {
                                        "array".to_string()
                                    }
                                }
                                "object" => "object".to_string(),
                                _ => kind.to_string(),
                            }
                        } else {
                            "complex".to_string()
                        }
                    }
                    _ => "unknown".to_string(),
                };

                let nullable = field_info
                    .get("nullable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let default_value = field_info.get("default").and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    Value::Bool(b) => Some(b.to_string()),
                    Value::Null => Some("null".to_string()),
                    _ => None,
                });

                columns.push(TableColumn {
                    name: field_name.clone(),
                    data_type: field_type,
                    nullable,
                    default_value,
                });
            }
        }

        // Sort columns by name for consistent output
        columns.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(TableSchema {
            table_name: table_name.to_string(),
            columns,
        })
    }
}

impl Tool for SurrealSchemaTool {
    const NAME: &'static str = "surreal_schema";

    type Error = SurrealError;
    type Args = SurrealSchemaArgs;
    type Output = String;

    fn name(&self) -> String {
        "surreal_schema".to_string()
    }

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "surreal_schema".to_string(),
            description: "Get column information and types for a SurrealDB table. Returns structured information about all columns including their names, data types, nullability, and default values.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "table_name": {
                        "type": "string",
                        "description": "The name of the table to get schema information for"
                    }
                },
                "required": ["table_name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate table name
        if args.table_name.trim().is_empty() {
            return Err(SurrealError::InvalidInput(
                "Table name cannot be empty".to_string(),
            ));
        }

        println!("Getting schema information for table: {}", args.table_name);

        // Construct the INFO TABLE query
        let query = format!("INFO FOR TABLE {}", args.table_name);

        // Execute the query using the shared function
        let result = execute_query(&self.config, &query).await?;

        // For now, just return the raw JSON result
        // TODO: Uncomment and fix the parsing logic below when needed
        // let schema = self._parse_table_info(&args.table_name, &result)?;
        // Ok(serde_json::to_string_pretty(&schema)?)

        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_surreal_schema_tool_creation() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSchemaTool::new(config);
        assert_eq!(tool.name(), "surreal_schema");
    }

    #[tokio::test]
    async fn test_tool_definition() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSchemaTool::new(config);
        let definition = tool.definition("".to_string()).await;

        assert_eq!(definition.name, "surreal_schema");
        assert!(!definition.description.is_empty());
        assert!(definition.parameters.get("properties").is_some());
    }

    #[tokio::test]
    async fn test_empty_table_name_error() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSchemaTool::new(config);
        let args = SurrealSchemaArgs {
            table_name: "".to_string(),
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SurrealError::InvalidInput(_)));
    }
}
