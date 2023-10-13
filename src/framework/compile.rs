// Here we combine the configurations into a set of prompts.
use crate::framework::structures::{
    AdditionalConfig, Attack, FullConfig, LoadedConfig, Mitigation,
};
use log::{debug, error, info, LevelFilter};

pub fn compile_runs(
    loaded_configs: Vec<LoadedConfig>,
    additional_configs: Vec<AdditionalConfig>,
    attacks: Vec<Attack>,
    mitigations: Vec<Vec<Mitigation>>,
) -> Result<Vec<FullConfig>, String> {
    let mut full_runs = Vec::new();

    for loaded in &loaded_configs {
        for attack in &attacks {
            for mits in &mitigations {
                for addl_conf in &additional_configs {
                    let full_run = FullConfig {
                        loaded_config: loaded.clone(),
                        additional_config: Some(addl_conf.clone()),
                        attack: attack.clone(),
                        mitigations: mits.clone(),
                    };
                    info!("Compiled full run: {:#?}", full_run);
                    full_runs.push(full_run);
                }
                let full_run = FullConfig {
                    loaded_config: loaded.clone(),
                    additional_config: None,
                    attack: attack.clone(),
                    mitigations: mits.clone(),
                };
                info!("Compiled full run: {:#?}", full_run);
                full_runs.push(full_run);
            }
        }
    }

    Ok(full_runs)
}
