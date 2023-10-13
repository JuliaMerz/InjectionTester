use futures::future::join_all;
use log::{debug, error, info, LevelFilter};
use pantry_rs::{error, interface};
use pantry_rs::{PantryClient, PantryError};
use std::thread::JoinHandle;

use crate::additional_configs;
use crate::attacks;
use crate::base_configs;
use crate::framework::structures::*;
use crate::mitigations;

fn validate_base_config(conf: BaseConfig) -> Result<(), String> {
    // check for sampler str
    // check for top p/freq/etc

    Ok(())
}
pub async fn load_base_configs(
    load_pub: bool,
    only_configs: Option<Vec<String>>,
    client: PantryClient,
) -> Result<Vec<LoadedConfig>, String> {
    let mut all_configs = base_configs::get_priv_base_configs();
    if load_pub {
        all_configs.extend(base_configs::get_pub_base_configs());
    }

    let s: String = "test".to_string() + &(32.to_string());

    let result = match only_configs {
        Some(names) => {
            let mut result = Vec::new();
            for name in names {
                match all_configs.get_mut(&name) {
                    Some(conf) => result.append(conf),
                    None => return Err(format!("BaseConfig {} not found", name)),
                }
            }
            result
        }
        None => all_configs.into_iter().flat_map(|(_, x)| x).collect(),
    };

    let mut futures = Vec::new();
    for conf in result.iter() {
        let uuid = client
            .get_or_download_llm(conf.llm_registry.clone())
            .await
            .map_err(|e| format!("Pantry download failure: {:?}", e))?;
        info!("get_or_downloaded {:?}", uuid);
        let samp = conf.samples.clone();
        let cl = client.clone();
        let reg = conf.llm_registry.clone();

        futures.push(tokio::spawn(async move {
            let llm = cl
                .await_download(uuid, |_| ())
                .await
                .map_err(|e| format!("Pantry await failure: {:?}", e))?;
            cl.load_llm(llm.uuid.clone()).await?;
            Ok(LoadedConfig {
                llm_registry: reg,
                llm: llm,
                samples: samp,
            })
        }));
    }

    let join_results: Result<Vec<Result<LoadedConfig, PantryError>>, tokio::task::JoinError> =
        join_all(futures.into_iter()).await.into_iter().collect();

    // let results = join_results
    //     .map_err(|e| format!("Join error: {:?}", e))?
    //     .into_iter()
    //     .collect();
    let res: Result<Vec<LoadedConfig>, PantryError> = join_results
        .map_err(|e| format!("Join error: {:?}", e))?
        .into_iter()
        .collect();

    res.map_err(|e| format!("Pantry other error: {:?}", e))
}

pub fn load_additional_configs(
    load_pub: bool,
    only_configs: Option<Vec<String>>,
) -> Result<Vec<AdditionalConfig>, String> {
    let mut all_configs = additional_configs::get_priv_additional_configs();
    if load_pub {
        all_configs.extend(additional_configs::get_pub_additional_configs());
    }

    match only_configs {
        Some(names) => {
            let mut result = Vec::new();
            for name in names {
                match all_configs.get_mut(&name) {
                    Some(conf) => result.append(conf),
                    None => return Err(format!("AdditionalConfig {} not found", name)),
                }
            }
            Ok(result)
        }
        None => Ok(all_configs.into_iter().flat_map(|(_, x)| x).collect()),
    }
}

pub fn load_attacks(
    load_pub: bool,
    only_configs: Option<Vec<String>>,
) -> Result<Vec<Attack>, String> {
    let mut all_configs = attacks::get_priv_attacks();
    if load_pub {
        all_configs.extend(attacks::get_pub_attacks());
    }

    match only_configs {
        Some(names) => {
            let mut result = Vec::new();
            for name in names {
                match all_configs.get_mut(&name) {
                    Some(conf) => result.append(conf),
                    None => return Err(format!("Attack {} not found", name)),
                }
            }
            Ok(result)
        }
        None => Ok(all_configs.into_iter().flat_map(|(_, x)| x).collect()),
    }
}

pub async fn load_mitigations(
    load_pub: bool,
    only_configs: Option<Vec<Vec<String>>>,
) -> Result<Vec<Vec<Mitigation>>, String> {
    let mut all_configs = mitigations::get_priv_mitigations();
    if load_pub {
        all_configs.extend(mitigations::get_pub_mitigations());
    }

    match only_configs {
        Some(vecs) => {
            let mut result = Vec::new();
            for names in vecs {
                let mut inner = Vec::new();
                for name in names {
                    match all_configs.get_mut(&name) {
                        Some(conf) => {
                            for m in conf.iter_mut() {
                                m.mitigator.run_once().await?;
                            }
                            inner.extend(conf.clone());
                        }
                        None => return Err(format!("Mitigation {} not found", name)),
                    }
                }
                result.push(inner);
            }
            Ok(result)
        }
        None => Ok(vec![all_configs.into_iter().flat_map(|(_, x)| x).collect()]),
    }
}
