// This file is auto-generated. Just add a config file.
use std::collections::HashMap;
use crate::framework::structures::BaseConfig;
mod public;
mod private;

pub fn get_pub_base_configs() -> HashMap<String, Vec<BaseConfig>> {
    let mut map = HashMap::new();
    map.insert("pub_".to_string()+"llama2", public::llama2::config());
    map.insert("pub_".to_string()+"example", public::example::config());
    map
}

pub fn get_priv_base_configs() -> HashMap<String, Vec<BaseConfig>> {
    let mut map = HashMap::new();
    map
}
