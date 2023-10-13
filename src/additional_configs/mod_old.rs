use std::collections::HashMap;

use crate::framework::structures::Mitigation;

pub fn get_mitigations() -> HashMap<String, Mitigation> {
    let mut map = HashMap::new();

    // Example:
    // map.insert("config1".to_string(), BaseConfig { /* ... */ });
    // ... add other configurations

    map
}
