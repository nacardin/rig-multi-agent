use std::{collections::BTreeMap, fmt::Display};

use rig::{client::CompletionClient, completion::Prompt, providers::xai::Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubQuestions {
    pub feature_requests: String,
    pub customers: String,
}

pub async fn map<S: Display>(
    client: &Client,
    question: &str,
    sub_agents: &BTreeMap<S, S>,
) -> SubQuestions {
    let sub_agents_string = sub_agents
        .into_iter()
        .map(|(name, desc)| format!("- \"{}\": \"{}\"", name, desc))
        .collect::<Vec<String>>()
        .join("\n");

    let agent1 = client
            .agent("grok-3-mini")
            .preamble(&format!(
                r#"
                You are a helpful assistant that can answer questions by delagating sub-questions to a sub-agent.
                Sub-agents are specialized in answering questions from a specific dataset.
                Sub-agents do not have access to data that they do not specialize in.

                You have access to the following sub-agents:
                {sub_agents_string}

                Please respond with a JSON object map with a key being the name of the sub-agent and a value being the sub-question to ask the sub-agent.
                "#,
            ))
            .build();

    // Prompt the model and print its response
    let response = agent1
        .prompt(question)
        .await
        .expect("Failed to prompt grok-3-mini");

    serde_json::from_str::<SubQuestions>(&response).unwrap()
}
