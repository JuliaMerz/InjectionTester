use std::collections::HashMap;

use crate::framework::structures::Attack;

// Define simple attacks here. For more complex attacks, define
// in a separate file and then import.

pub fn get_attacks() -> HashMap<String, Vulnerability> {
    let mut map = HashMap::new();

    // Example:
    // map.insert("config1".to_string(), BaseConfig { /* ... */ });
    // ... add other configurations

    map
}
