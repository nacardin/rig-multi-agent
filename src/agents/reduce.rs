use rig::{client::CompletionClient, completion::Prompt, providers::xai::Client};

pub async fn reduce(client: &Client, question: &str, data: Vec<String>) -> String {
    let data_string = data.join("\n\n");

    let agent1 = client
            .agent("grok-3-mini")
            .preamble(&format!(
                r#"
                You are a helpful assistant that can answer questions by based on data provided below.
               Use only the provided data, but use your own knowledge to analyze and determine what it means and come up with conclusions that would be useful to a business decision maker.

                {data_string}

                "#
            ))
            .build();

    agent1
        .prompt(question)
        .await
        .expect("Failed to prompt grok-3-mini")
}
