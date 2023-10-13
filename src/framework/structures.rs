use crate::framework::mitigations::MitigationTechnique;
use crate::framework::vulnerability::ScoringStrategy;
use colored::Colorize;
use pantry_rs::interface::{LLMConnectorType, LLMRegistryEntry, LLMStatus};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{self, write};

#[derive(Clone, Debug)]
pub struct BaseConfig {
    // all llm_registry_entries must have flexible topp/topk/etc values.
    pub llm_registry: LLMRegistryEntry,
    pub samples: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct LoadedConfig {
    // all llm_registry_entries must have flexible topp/topk/etc values.
    pub llm_registry: LLMRegistryEntry,
    pub llm: LLMStatus,
    pub samples: Vec<(String, String)>,
}

/// rustformers/llm supports considerably more complex samplers, but for the sake of comparability
/// we're currently sticking to these six. The OpeanAI comptabile ones also the ones used by the
/// original promptinject paper.
///
/// OpenAI models will support:
/// - temperature
/// - top_p
/// - presence
/// - frequency
///
#[derive(Clone, Debug)]
pub struct AdditionalConfig {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    ///WARNING: THIS DOES NOTHING FOR OPENAI
    pub top_k: Option<i32>,
    pub presence: Option<f32>,
    pub frequency: Option<f32>,
    ///WARNING: THIS DOES NOTHING FOR OPENAI
    pub repetition: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct Vulnerability {
    pub vulnerability_name: String,
    pub vulnerability_description: String,
    pub adversarial: bool,
    /// This is what we include in the LLM prompt.
    pub setup_secret: String,
    pub scoring_strategy: ScoringStrategy,
}

#[derive(Clone, Debug)]
pub struct Attack {
    pub vulnerability: Vulnerability,
    pub attack_label: String,
    pub attack_description: String,
    pub attack_phrase: String,
}

#[derive(Clone, Debug)]
pub struct Mitigation {
    pub label: String,
    pub mitigator: Box<dyn MitigationTechnique>,
}

#[derive(Clone, Debug)]
pub struct FullConfig {
    pub loaded_config: LoadedConfig,
    pub additional_config: Option<AdditionalConfig>,
    pub attack: Attack,
    pub mitigations: Vec<Mitigation>,
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    pub response: String,
    pub attack_success: bool,
    pub evaluation: String,
}

#[derive(Clone, Debug)]
pub struct CompletedRun {
    pub config: FullConfig,
    pub result: Result<Evaluation, String>,
}

pub struct RunSet(pub Vec<CompletedRun>);

//setsup:
//
//baseconfig system prompt
//attack secret
//sample sample
//sample sample
//
// THEN feed in attack.
impl FullConfig {
    pub fn get_system_prompt(&self) -> Result<String, String> {
        let mut system_prompt = self
            .loaded_config
            .llm_registry
            .session_parameters
            .get("system_prompt")
            .unwrap_or(&Value::String("".into()))
            .as_str()
            .unwrap_or("".into())
            .to_string();

        system_prompt = system_prompt + "\n" + &self.attack.vulnerability.setup_secret + "\n";

        let def = Value::String("".into());
        let pre_prompt = self
            .loaded_config
            .llm_registry
            .parameters
            .get("pre_prompt")
            .unwrap_or(&def)
            .as_str()
            .unwrap_or("".into());
        let post_prompt = self
            .loaded_config
            .llm_registry
            .parameters
            .get("post_prompt")
            .unwrap_or(&def)
            .as_str()
            .unwrap_or("".into());

        for samp in self.loaded_config.samples.iter() {
            system_prompt =
                system_prompt + pre_prompt + &samp.0 + "\n" + post_prompt + &samp.1 + "\n";
        }

        Ok(system_prompt)
    }

    pub fn get_prompt_parameters(&self) -> Result<HashMap<String, Value>, String> {
        match self.loaded_config.llm_registry.connector_type {
            LLMConnectorType::LLMrs => {
                let mut map: HashMap<String, Value> = HashMap::new();
                //we need to build a sampler string here
                let sampler_string = match self.additional_config.clone() {
                    Some(conf) => {
                        let top_p = conf.top_p.unwrap_or(0.95);
                        let top_k = conf.top_k.unwrap_or(40);
                        let temperature = conf.temperature.unwrap_or(0.8);
                        let repetition_penalty = conf.repetition.unwrap_or(1.3);

                        let fp_str = if conf.presence.is_some() || conf.frequency.is_some() {
                            let presence = conf.presence.unwrap_or(0.0);
                            let frequency = conf.frequency.unwrap_or(0.0);
                            format!(
                                " freqpresnce:presence_penalty={}:frequency_penalty={}",
                                presence, frequency
                            )
                        } else {
                            format!("")
                        };

                        format!("repetition:penalty={}:last_n=64 topk:k={}:min_keep=1 topp:p={}:min_keep=1 temperature:{}{}", repetition_penalty, top_k, top_p, temperature, fp_str)
                    }
                    None => {
                        format!("repetition:penalty=1.3:last_n=64 topk:k=40:min_keep=1 topp:p=0.95:min_keep=1 temperature:0.8")
                    }
                };

                map.insert("sampler_string".into(), sampler_string.into());

                Ok(map)
            }
            LLMConnectorType::OpenAI => {
                let mut map: HashMap<String, Value> = HashMap::new();
                // We only have parameters on prompting with openai so we need sys prompt
                // on prompt
                map.insert(
                    "system_prompt".into(),
                    Value::String(self.get_system_prompt()?),
                );

                if let Some(conf) = self.additional_config.clone() {
                    if let Some(top_k) = conf.top_k {
                        map.insert("top_k".into(), top_k.into());
                    }
                    if let Some(top_p) = conf.top_p {
                        map.insert("top_p".into(), top_p.into());
                    }
                    if let Some(presence_penalty) = conf.presence {
                        map.insert("presence_penalty".into(), presence_penalty.into());
                    }
                    if let Some(frequency_penalty) = conf.frequency {
                        map.insert("frequency_penalty".into(), frequency_penalty.into());
                    }
                }
                Ok(map)
            }
            _ => {
                return Err(format!("Invalid/Unsupported connector type for {:#?}", self).into());
            }
        }
    }

    pub fn get_session_parameters(&self) -> Result<HashMap<String, Value>, String> {
        match self.loaded_config.llm_registry.connector_type {
            LLMConnectorType::LLMrs => {
                let mut map = HashMap::new();
                map.insert(
                    "system_prompt".into(),
                    Value::String(self.get_system_prompt()?),
                );
                //we need to build a sampler string here
                Ok(map)
            }
            LLMConnectorType::OpenAI => {
                let map = HashMap::new();
                Ok(map)
            }
            _ => {
                return Err(format!("Invalid/Unsupported connector type for {:#?}", self).into());
            }
        }
    }

    pub fn get_attack_prompt(&self) -> Result<String, String> {
        Ok(self.attack.attack_phrase.clone())
    }
}

impl fmt::Display for FullConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /// Include: llm name, llm id
        /// addl config params
        /// Attack name
        /// mitigation names
        writeln!(
            f,
            "{} {} — {}",
            "[LLM]".green(),
            self.loaded_config.llm.name.bold(),
            self.loaded_config.llm.id.bold()
        );
        write!(f, "{}", "[Config] ".green());
        if self.additional_config.is_some() {
            match self.loaded_config.llm_registry.connector_type {
                LLMConnectorType::LLMrs => {
                    let empty_value_string = Value::String("".into());
                    writeln!(
                        f,
                        "{}",
                        self.get_prompt_parameters()
                            .unwrap_or(HashMap::new())
                            .get("sampler_string")
                            .unwrap_or(&empty_value_string)
                            .as_str()
                            .unwrap_or(""),
                        // self.loaded_config.llm.name, self.loaded_config.llm.id
                    );
                }
                LLMConnectorType::OpenAI => {
                    writeln!(
                        f,
                        "TopP {} | Temperature {} | Frequency {} | Presence {}",
                        self.additional_config.clone().unwrap().top_p.unwrap_or(1.),
                        self.additional_config
                            .clone()
                            .unwrap()
                            .temperature
                            .unwrap_or(1.),
                        self.additional_config
                            .clone()
                            .unwrap()
                            .frequency
                            .unwrap_or(0.),
                        self.additional_config
                            .clone()
                            .unwrap()
                            .presence
                            .unwrap_or(0.),
                    );
                }
                _ => {
                    writeln!(f, "[ERROR] Unknown LLM Connector.");
                }
            }
        } else {
            match self.loaded_config.llm_registry.connector_type {
                LLMConnectorType::LLMrs => {
                    writeln!(f, "(default) repetition:penalty=1.3:last_n=64 topk:k=40:min_keep=1 topp:p=0.95:min_keep=1 temperature:0.8");
                }
                LLMConnectorType::OpenAI => {
                    writeln!(
                        f,
                        "(default) TopP 1 | Temperature 1 | Frequency 0 | Presence 0",
                    );
                }
                _ => {
                    writeln!(f, "(ERROR) Unknown LLM Connector.");
                }
            }
        }

        // Attack
        write!(f, "{}", "[Attack] ".green());
        write!(
            f,
            "{}{} {}— {}{} \n",
            "V: ".bold(),
            self.attack.vulnerability.vulnerability_name,
            "A: ".bold(),
            self.attack.attack_label,
            match self.attack.vulnerability.adversarial {
                true => "".cyan(),
                false => "(non-adversarial) ".cyan().bold(),
            }
        );
        writeln!(
            f,
            "Vulnerability: {}",
            self.attack.vulnerability.vulnerability_description
        );
        writeln!(f, "Attack: {}", self.attack.attack_description);

        // Mitigations:
        write!(f, "{}", "[Mitigations] ".green());
        for m in self.mitigations.iter() {
            write!(f, "{} | ", m.label);
        }
        write!(f, "\n");

        Ok(())
    }
}

impl fmt::Display for CompletedRun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.config);
        writeln!(f, "{}", "===RESULT===".bold().green());
        match self.result.clone() {
            Ok(eval) => {
                writeln!(
                    f,
                    "{} {}",
                    "[Request]".green(),
                    self.config.get_attack_prompt().unwrap_or("".into())
                );
                writeln!(f, "{} {}", "[Response]".green(), eval.response.clone());
                writeln!(
                    f,
                    "{} {} {}",
                    "[Evaluation]".green(),
                    match (
                        eval.attack_success,
                        // If the LLM was supposed to give itself away, it's non-adversarial and we
                        // haven't "won".
                        self.config.attack.vulnerability.adversarial
                    ) {
                        (true, true) => "(success)".green(),
                        (false, _) => "(failure)".red(),
                        (true, false) => "(success — non-adversarial)".cyan(),
                    },
                    eval.evaluation,
                );
            }
            Err(e) => {
                writeln!(f, "{} {}", "[ERROR]".red(), e);
            }
        }
        Ok(())
    }
}

impl fmt::Display for RunSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "Vec<CompletedRun>: No result.");
            return Ok(());
        }
        let first = self.0.first().unwrap();
        write!(f, "{}", first.config);
        writeln!(f, "{}", "===RESULT===".bold().green());
        writeln!(
            f,
            "{} {}/{}, {}",
            "Success Rate:".bold(),
            self.0
                .iter()
                .map(|se| match &se.result {
                    Ok(eval) => eval.attack_success.into(),
                    Err(_) => 0,
                })
                .reduce(|acc, x| acc + x)
                .unwrap_or(-1),
            self.0.len(),
            self.0
                .iter()
                .map(|se| match &se.result {
                    Ok(eval) => eval.attack_success.into(),
                    Err(_) => 0.,
                })
                .reduce(|acc, x| acc + x)
                .unwrap_or(-1.)
                / i32::try_from(self.0.len()).unwrap() as f32
        );
        writeln!(
            f,
            "{} {}",
            "[Request]".green(),
            first
                .config
                .get_attack_prompt()
                .unwrap_or("".into())
                .italic()
        );
        for run in self.0.iter() {
            match run.result.clone() {
                Ok(eval) => {
                    writeln!(f, "{} {}", "[Response]".green(), eval.response.clone());
                    writeln!(
                        f,
                        "{} {} {}",
                        "[Evaluation]".green(),
                        match (
                            eval.attack_success,
                            // If the LLM was supposed to give itself away, it's non-adversarial and we
                            // haven't "won".
                            run.config.attack.vulnerability.adversarial
                        ) {
                            (true, true) => "(success)".green(),
                            (false, _) => "(failure)".red(),
                            (true, false) => "(success — non-adversarial)".cyan(),
                        },
                        eval.evaluation,
                    );
                }
                Err(e) => {
                    writeln!(f, "{} {}", "[ERROR]".red(), e);
                }
            }
        }
        Ok(())
    }
}
