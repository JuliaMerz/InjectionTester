use crate::framework::structures::AdditionalConfig;

pub fn config() -> Vec<AdditionalConfig> {
    let conf = AdditionalConfig {
        frequency: None,
        presence: None,
        repetition: Some(1.2),
        top_p: Some(0.9),
        top_k: Some(40),
        temperature: Some(0.7),
    };
    vec![conf]
}
