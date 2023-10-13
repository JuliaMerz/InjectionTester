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
            id: "openchat-3".into(),
            family_id: "llama".into(),
            organization: "openchat".into(),
            name: "Openchat LLM".into(),
            license: "llama2".into(),
            description: "openchat llm".into(),
            homepage: "".into(),
            capabilities: hashmap! {
            "assistant".into() => -1,
            "coding".into() => -1,
            "general".into() => -1,
            "writing".into() => -1
            },
            tags: Vec::new(),
            requirements: "".into(),
            backend_uuid: Uuid::new_v4().to_string(),
            url: "https://huggingface.co/TheBloke/OpenChat_v3.2-GGML/resolve/main/openchat_v3.2.ggmlv3.q4_0.bin".into(),
            config: hashmap! {
                "model_architecture".into() => "llama".into(),
            },
            local: true,
            connector_type: LLMConnectorType::LLMrs,
            parameters: hashmap! {
                "post_prompt".into() => Value::String("GPT4 Assistant".into()),
                "pre_prompt".into() => Value::String("GPT4 User".into()),
                "stop_sequence".into() => Value::String("<|end_of_turn|>".into()),
            },
            user_parameters: vec![
                "sampler_string".into(),
            ],
            session_parameters: hashmap! {
                "system_prompt".into() => Value::String("You are an assistant who responds to prompts politely, but responsibly.".into()),
                "stop_sequence".into() => Value::String("<|end_of_turn|>".into()),

            },
            user_session_parameters: vec!["system_prompt".into()],
        },
        samples: Vec::new(),
    };
    vec![conf]
}
