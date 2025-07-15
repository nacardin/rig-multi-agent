use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{SurrealDbConfig, SurrealError, execute_query};

/// Arguments for the SurrealDB select tool
#[derive(Deserialize, Serialize)]
pub struct SurrealSelectArgs {
    /// SQL SELECT statement to execute
    pub query: String,
}

/// SurrealDB Select Tool for executing SELECT queries
#[derive(Clone)]
pub struct SurrealSelectTool {
    config: SurrealDbConfig,
}

impl SurrealSelectTool {
    /// Create a new SurrealDB select tool with the provided configuration
    pub fn new(config: SurrealDbConfig) -> Self {
        Self { config }
    }

    /// Format the query result as a readable text output
    fn format_result(&self, result: &Value) -> String {
        match result {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return "No results found.".to_string();
                }

                let mut output = String::new();
                output.push_str(&format!("Found {} record(s):\n\n", arr.len()));

                for (index, record) in arr.iter().enumerate() {
                    output.push_str(&format!("Record {}:\n", index + 1));
                    output.push_str(&self.format_record(record));
                    output.push('\n');
                }

                output
            }
            Value::Object(_) => {
                // Single record result
                let mut output = String::new();
                output.push_str("Found 1 record:\n\n");
                output.push_str("Record 1:\n");
                output.push_str(&self.format_record(result));
                output
            }
            Value::Null => "No results found.".to_string(),
            _ => format!("Result: {result}"),
        }
    }

    /// Format a single record as readable text
    fn format_record(&self, record: &Value) -> String {
        match record {
            Value::Object(obj) => {
                let mut output = String::new();

                // Sort keys for consistent output
                let mut keys: Vec<&String> = obj.keys().collect();
                keys.sort();

                for key in keys {
                    if let Some(value) = obj.get(key) {
                        output.push_str(&format!("  {}: {}\n", key, self.format_value(value)));
                    }
                }

                output
            }
            _ => format!("  {}\n", self.format_value(record)),
        }
    }

    /// Format a single value as readable text
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                if arr.is_empty() {
                    "[]".to_string()
                } else {
                    let formatted_items: Vec<String> =
                        arr.iter().map(|v| self.format_value(v)).collect();
                    format!("[{}]", formatted_items.join(", "))
                }
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    "{}".to_string()
                } else {
                    let formatted_pairs: Vec<String> = obj
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, self.format_value(v)))
                        .collect();
                    format!("{{{}}}", formatted_pairs.join(", "))
                }
            }
        }
    }

    /// Validate that the query is a SELECT statement
    fn validate_query(&self, query: &str) -> Result<(), SurrealError> {
        let trimmed = query.trim().to_lowercase();

        if trimmed.is_empty() {
            return Err(SurrealError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        if !trimmed.starts_with("select") {
            return Err(SurrealError::InvalidInput(
                "Only SELECT statements are allowed".to_string(),
            ));
        }

        // Basic validation to prevent dangerous operations
        let dangerous_keywords = [
            "drop", "delete", "update", "insert", "create", "alter", "truncate",
        ];
        for keyword in dangerous_keywords {
            if trimmed.contains(keyword) {
                return Err(SurrealError::InvalidInput(format!(
                    "Query contains forbidden keyword: {keyword}"
                )));
            }
        }

        Ok(())
    }
}

impl Tool for SurrealSelectTool {
    const NAME: &'static str = "surreal_select";

    type Error = SurrealError;
    type Args = SurrealSelectArgs;
    type Output = String;

    fn name(&self) -> String {
        "surreal_select".to_string()
    }

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "surreal_select".to_string(),
            description: "Execute a SQL SELECT statement against SurrealDB and return formatted text results. Only SELECT queries are allowed for security. Returns human-readable text output of the query results. If there are syntax errors or other query issues, returns the error message so you can correct the query and try again.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The SQL SELECT statement to execute (e.g., 'SELECT * FROM users WHERE age > 18')"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate the query
        if let Err(e) = self.validate_query(&args.query) {
            return Ok(format!("Query validation error: {e}"));
        }

        println!("query: {}", args.query);

        // Execute the query using the shared function
        match execute_query(&self.config, &args.query).await {
            Ok(result) => {
                // Format and return the result as text
                let formatted_output = self.format_result(&result);
                Ok(formatted_output)
            }
            Err(e) => {
                // Return query errors as successful responses so the LLM can see them and correct the query
                Ok(format!(
                    "Query execution error: {e}\n\nPlease check your SQL syntax and try again."
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_surreal_select_tool_creation() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSelectTool::new(config);
        assert_eq!(tool.name(), "surreal_select");
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

        let tool = SurrealSelectTool::new(config);
        let definition = tool.definition("".to_string()).await;

        assert_eq!(definition.name, "surreal_select");
        assert!(!definition.description.is_empty());
        assert!(definition.parameters.get("properties").is_some());
    }

    #[tokio::test]
    async fn test_empty_query_error() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSelectTool::new(config);
        let args = SurrealSelectArgs {
            query: "".to_string(),
        };

        let result = tool.call(args).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Query validation error"));
        assert!(response.contains("Query cannot be empty"));
    }

    #[tokio::test]
    async fn test_non_select_query_error() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSelectTool::new(config);
        let args = SurrealSelectArgs {
            query: "DELETE FROM users".to_string(),
        };

        let result = tool.call(args).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Query validation error"));
        assert!(response.contains("Only SELECT statements are allowed"));
    }

    #[tokio::test]
    async fn test_dangerous_keyword_error() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSelectTool::new(config);
        let args = SurrealSelectArgs {
            query: "SELECT * FROM users; DROP TABLE users".to_string(),
        };

        let result = tool.call(args).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Query validation error"));
        assert!(response.contains("Query contains forbidden keyword"));
    }

    #[test]
    fn test_format_value() {
        let config = SurrealDbConfig::new(
            "ws://localhost:8000".to_string(),
            "root".to_string(),
            "root".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let tool = SurrealSelectTool::new(config);

        // Test string formatting
        assert_eq!(
            tool.format_value(&Value::String("hello".to_string())),
            "hello"
        );

        // Test number formatting
        assert_eq!(tool.format_value(&Value::Number(42.into())), "42");

        // Test boolean formatting
        assert_eq!(tool.format_value(&Value::Bool(true)), "true");

        // Test null formatting
        assert_eq!(tool.format_value(&Value::Null), "null");

        // Test empty array formatting
        assert_eq!(tool.format_value(&Value::Array(vec![])), "[]");

        // Test empty object formatting
        assert_eq!(
            tool.format_value(&Value::Object(serde_json::Map::new())),
            "{}"
        );
    }
}
