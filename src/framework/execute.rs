use crate::framework::structures::{CompletedRun, Evaluation, FullConfig};
use crate::framework::vulnerability::evaluate_vulnerability;
use futures::future::join_all;
use futures::prelude::*;
use futures::StreamExt;
use log::{debug, error, info, warn, LevelFilter};
use pantry_rs::interface::{LLMConnectorType, LLMEventInternal};
use pantry_rs::PantryClient;

pub async fn execute_run(
    pantry_client: &PantryClient,
    full_config: FullConfig,
) -> Result<Evaluation, String> {
    let session_parameters = full_config.get_session_parameters()?;
    info!(
        "Creating LLM session with parameters: {:#?}",
        session_parameters
    );
    let llm_session = pantry_client
        .create_session(session_parameters.clone())
        .await
        .map_err(|e| format!("LLM Session Failure: {:?}", e))?;
    let prompt_parameters = full_config.get_prompt_parameters()?;
    info!(
        "Creating LLM prompt with parameters: {:#?}",
        session_parameters.clone()
    );

    let compiled_prompt = full_config.get_attack_prompt()?;

    let mut stream = llm_session
        .prompt_session(compiled_prompt, prompt_parameters)
        .await
        .map_err(|e| format!("LLM Prompt Failure: {:?}", e))?;
    let mut output: String = "".into();
    while let Some(ev) = stream.next().await {
        info!("{:?}", ev.event);
        match ev.event {
            LLMEventInternal::PromptProgress { previous, next } => {
                output = previous + &next;
            }
            LLMEventInternal::PromptCompletion { previous } => {
                output = previous;
            }
            LLMEventInternal::PromptError { message } => {
                error!("Error returning from llm: {}", message);
                return Err(message);
            }
            unknown => {
                warn!("Unknown event: {:?}", unknown);
            }
        }
    }

    evaluate_vulnerability(&full_config.attack.vulnerability, output).await
}

pub async fn execute_full_runs(
    pantry_client: PantryClient,
    full_runs: Vec<FullConfig>,
    n: i32,
) -> Result<Vec<Vec<CompletedRun>>, String> {
    let mut config_joins = Vec::new();
    for full_config in full_runs.into_iter() {
        let client = pantry_client.clone();
        let full_conf = full_config.clone();
        // We're wrapping the tokio spawn in an async here so that buffer_unordered
        // lets us run two at a time—otherwise the tokio spawn triggers before
        // await gets called internally.
        config_joins.push(async {
            tokio::spawn(async move {
                let mut config_set = Vec::new();
                for i in 0..n {
                    let res = execute_run(&client, full_conf.clone()).await;
                    config_set.push(CompletedRun {
                        config: full_conf.clone(),
                        result: res,
                    });
                }
                config_set
            })
            .await
        });
    }
    let stream = futures::stream::iter(config_joins).buffer_unordered(2);

    // wait for all futures to complete
    let join_results: Result<Vec<Vec<CompletedRun>>, tokio::task::JoinError> =
        stream.collect::<Vec<_>>().await.into_iter().collect();
    // let join_results: Result<Vec<Vec<CompletedRun>>, tokio::task::JoinError> =
    //     join_all(config_joins.into_iter())
    //         .await
    //         .into_iter()
    //         .collect();
    let completed_runs: Result<Vec<Vec<CompletedRun>>, String> =
        join_results.map_err(|e| format!("Join error: {:?}", e));

    // completed_runs.push(CompletedRun {
    //     config: full_config,
    //     result: res,
    // });
    completed_runs
}
