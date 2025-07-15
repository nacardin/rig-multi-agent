pub mod agents;
pub mod config;
pub mod surreal;

use std::collections::BTreeMap;

use config::Config;
use rig::providers::xai;
use surreal::SurrealSelectTool;

#[tokio::main]
async fn main() {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            eprintln!("Please check your .env file or environment variables.");
            eprintln!("Copy .env.example to .env and fill in your values.");
            std::process::exit(1);
        }
    };

    let xai_client = xai::Client::new(&config.xai_api_key);

    let question = r#"
        Which feature requests should I prioritize to satisfy my highest paying customers?
        Analyze the tone of customer's message feature requests to determine their urgency.
        Combine the urgency of the FR with the value of each customer to prioritize feature requests.
        "#;

    let mut agents = BTreeMap::new();

    agents.insert("feature_requests", "This agent specializes in finding incoming support tickets or feedback logs with feature requests. Do not ask it about customer data other than identifiers.");
    agents.insert("customers", "This agent specializes in finding customers and their Annual Recurring Revenue (ARR) in USD. Do not ask it about feature requests.");

    let sq = crate::agents::map::map(&xai_client, question, &agents).await;

    println!("sub questions: {sq:?}");

    let fr_resp = crate::agents::query::question(
        &xai_client,
        &sq.feature_requests,
        "feature_requests",
        "This table captures incoming support tickets or feedback logs with feature requests.",
        &config.surreal_config,
    )
    .await;
    let cust_resp = crate::agents::query::question(
        &xai_client,
        &sq.customers,
        "customers",
        "This table lists customers and their Annual Recurring Revenue (ARR) in USD.",
        &config.surreal_config,
    )
    .await;

    println!("fr_resp: {fr_resp}");
    println!("cust_resp: {cust_resp}");

    let mut data: Vec<String> = Vec::new();
    data.push(fr_resp);
    data.push(cust_resp);

    let answer = crate::agents::reduce::reduce(&xai_client, question, data).await;

    println!("answer: {answer}");
}
