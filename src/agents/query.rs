use rig::{client::CompletionClient, completion::Prompt, providers::xai::Client};

use crate::{
    SurrealSelectTool,
    config::SurrealConfig,
    surreal::{SurrealDbConfig, SurrealSchemaTool},
};

pub async fn question(
    xai_client: &Client,
    question: &str,
    table: &str,
    table_context: &str,
    surreal_config: &SurrealConfig,
) -> String {
    let surreal_db_config = SurrealDbConfig::new(
        surreal_config.host.clone(),
        surreal_config.username.clone(),
        surreal_config.password.clone(),
        surreal_config.namespace.clone(),
        surreal_config.database.clone(),
    );

    // Create tools
    let schema_tool = SurrealSchemaTool::new(surreal_db_config.clone());
    let select_tool = SurrealSelectTool::new(surreal_db_config);

    let agent_builder = xai_client
        .agent("grok-3-mini")
        .preamble(&format!(r#"
            You are a helpful assistant that can answer questions from the {table} table.
            {table_context}

            You have access to tools for database operations:
            - surreal_schema: Get schema information for database tables
            - surreal_select: Execute SELECT queries to retrieve data

            Use the surreal_select tool to retrieve data from the {table} table.

            IMPORTANT: If a query fails with a syntax error or other issue, the tool will return an error message instead of failing. Read the error message carefully and correct your query syntax before trying again. Common issues include:
            - Missing quotes around string values
            - Incorrect parentheses matching
            - Invalid SQL syntax

            Use CONTAINS operator in WHERE clause to partial match on string values.

            The tools connect to a SurrealDB instance. See SQL syntax here https://surrealdb.com/docs/surrealql/statements/select, https://surrealdb.com/docs/surrealql/clauses/where, https://surrealdb.com/docs/surrealql/datamodel/strings.

            Mention the IDs of which rows were used to generate the response.
            "#));

    let agent2 = agent_builder
        .tool(schema_tool.clone())
        .tool(select_tool.clone())
        .build();

    agent2
        .prompt(question)
        .multi_turn(10)
        .await
        .expect("Failed to prompt grok-3-mini")
}
