use crate::framework::structures::BaseConfig;
use serde_json::Value;

use maplit::hashmap;
use pantry_rs::interface::{LLMConnectorType, LLMRegistryEntry};
use uuid::Uuid;

/// This function returns a vector. They'll be treated individually,
/// but the Vec return ensures that configs can define multiple
/// configs in a set if they choose to do so.
pub fn config() -> Vec<BaseConfig> {
    let conf = BaseConfig {
        llm_registry: LLMRegistryEntry {
            id: "gpt-3.5-turbo".into(),
            family_id: "gpt".into(),
            organization: "openai".into(),
            name: "GPT 3.5".into(),
            license: "commercial".into(),
            description: "Direct line to OpenAI's ChatGPT".into(),
            homepage: "https://platform.openai.com/docs/models/gpt-3-5".into(),
            capabilities: hashmap! {
            "assistant".into() => -1,
            "coding".into() => -1,
            "general".into() => -1,
            "writing".into() => -1
            },
            tags: Vec::new(),
            requirements: "".into(),
            backend_uuid: Uuid::new_v4().to_string(),
            url: "".into(),
            config: hashmap! {
                "model_architecture".into() => "gpt-3.5-turbo".into(),
            },
            local: true,
            connector_type: LLMConnectorType::OpenAI,
            parameters: hashmap! {},
            user_parameters: vec![
                "top_p".into(),
                "temperature".into(),
                "frequency_penalty".into(),
                "presence_penalty".into(),
            ],
            session_parameters: hashmap! {
                "system_prompt".into() => Value::String("You are an assistant who responds to prompts politely, but responsibly.".into()),

            },
            user_session_parameters: vec!["system_prompt".into()],
        },
        samples: Vec::new(),
    };
    vec![conf]
}
