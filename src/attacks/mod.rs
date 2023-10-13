// This file is auto-generated. Just add a config file.
use std::collections::HashMap;
use crate::framework::structures::Attack;
mod vulnerabilities;
mod public;
mod private;

pub fn get_pub_attacks() -> HashMap<String, Vec<Attack>> {
    let mut map = HashMap::new();
    map.insert("pub_".to_string()+"gov_secret", public::gov_secret::config());
    map.insert("pub_".to_string()+"example", public::example::config());
    map
}

pub fn get_priv_attacks() -> HashMap<String, Vec<Attack>> {
    let mut map = HashMap::new();
    map
}
