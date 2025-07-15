//! SurrealDB tools and utilities for the rig framework
//!
//! This module provides tools for interacting with SurrealDB databases,
//! including schema inspection and query execution capabilities.

use std::error::Error as StdError;
use std::fmt;

pub mod schema;
pub mod select;

/// Configuration for SurrealDB connection
#[derive(Clone, Debug)]
pub struct SurrealDbConfig {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub db: String,
    pub namespace: String,
}

impl SurrealDbConfig {
    /// Create a new SurrealDB configuration
    pub fn new(url: String, user: String, pass: String, db: String, namespace: String) -> Self {
        Self {
            url,
            user,
            pass,
            db,
            namespace,
        }
    }
}

/// Common error type for SurrealDB operations
#[derive(Debug)]
pub enum SurrealError {
    ConnectionError(String),
    QueryError(String),
    SerializationError(serde_json::Error),
    InvalidInput(String),
}

impl fmt::Display for SurrealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SurrealError::ConnectionError(msg) => write!(f, "Connection error: {msg}"),
            SurrealError::QueryError(msg) => write!(f, "Query error: {msg}"),
            SurrealError::SerializationError(err) => write!(f, "Serialization error: {err}"),
            SurrealError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl StdError for SurrealError {}

impl From<serde_json::Error> for SurrealError {
    fn from(err: serde_json::Error) -> Self {
        SurrealError::SerializationError(err)
    }
}

/// Common database connection and query execution functionality
pub(crate) async fn execute_query(
    config: &SurrealDbConfig,
    query: &str,
) -> Result<serde_json::Value, SurrealError> {
    use surrealdb::{Surreal, engine::remote::ws::Wss};

    // Connect to SurrealDB
    let db = Surreal::new::<Wss>(&config.url)
        .await
        .map_err(|e| SurrealError::ConnectionError(e.to_string()))?;

    // Sign in as root user
    db.signin(surrealdb::opt::auth::Root {
        username: &config.user,
        password: &config.pass,
    })
    .await
    .map_err(|e| SurrealError::ConnectionError(e.to_string()))?;

    // Use the specified namespace and database
    db.use_ns(&config.namespace)
        .use_db(&config.db)
        .await
        .map_err(|e| SurrealError::ConnectionError(e.to_string()))?;

    // Execute the query
    let mut result = db
        .query(query)
        .await
        .map_err(|e| SurrealError::QueryError(e.to_string()))?;

    // Take the first result from the query response
    let query_result: surrealdb::Value = result
        .take(0)
        .map_err(|e| SurrealError::QueryError(e.to_string()))?;

    // Convert to serde_json::Value
    let json_value =
        serde_json::to_value(query_result).map_err(SurrealError::SerializationError)?;

    Ok(json_value)
}

// Re-export the tools for convenience
pub use schema::SurrealSchemaTool;
pub use select::SurrealSelectTool;
