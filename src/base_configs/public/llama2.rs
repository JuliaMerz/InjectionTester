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
            id: "Llama-2-13b-chat-hf".into(),
            family_id: "llama".into(),
            organization: "meta".into(),
            name: "Llama2 13b".into(),
            license: "llama2".into(),
            description: "llama 2 ggml'd by the bloke".into(),
            homepage: "https://huggingface.co/TheBloke/Llama-2-13B-GGML".into(),
            capabilities: hashmap! {
            "assistant".into() => -1,
            "coding".into() => -1,
            "general".into() => -1,
            "writing".into() => -1
            },
            tags: Vec::new(),
            requirements: "".into(),
            backend_uuid: Uuid::new_v4().to_string(),
            url: "https://huggingface.co/TheBloke/Llama-2-13B-GGML/resolve/main/llama-2-13b.ggmlv3.q4_0.bin".into(),
            config: hashmap! {
                "model_architecture".into() => "llama".into(),
            },
            local: true,
            connector_type: LLMConnectorType::LLMrs,
            parameters: hashmap! {
                "post_prompt".into() => Value::String("[[INST]]".into()),
                "pre_prompt".into() => Value::String("[[/INST]]".into()),
                "stop_sequence".into() => Value::String("</s>".into()),
            },
            user_parameters: vec![
                "sampler_string".into(),
            ],
            session_parameters: hashmap! {
                "system_prompt".into() => Value::String("[INST] <<SYS>>
You are a helpful, respectful and honest assistant. Always answer as helpfully as possible, while being safe.  Your answers should not include any harmful, unethical, racist, sexist, toxic, dangerous, or illegal content. Please ensure that your responses are socially unbiased and positive in nature. If a question does not make any sense, or is not factually coherent, explain why instead of answering something not correct. If you don't know the answer to a question, please don't share false information.
<</SYS>>".into()),
                "stop_sequence".into() => Value::String("</s>".into()),

            },
            user_session_parameters: vec!["system_prompt".into()],
        },
        samples: Vec::new(),
    };
    vec![conf]
}
