use crate::framework::{compile, execute, load, structures};
use clap::Parser;
use colored::Colorize;
use dotenv::dotenv;
use env_logger::Builder;
use log::{debug, error, info, LevelFilter};
use uuid::Uuid;

pub mod additional_configs;
pub mod attacks;
pub mod base_configs;
pub mod framework;
pub mod mitigations;

// pub fn compile_additional_configs()
//

/// Runs the injectonator frameowkr
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Cuts out all public config files from consideration (even if they're named later)
    ///
    /// This is most useful when you're just letting "all" run and want to only run your private
    /// setup.
    #[arg(short = 'p', long)]
    no_public: bool,

    /// Dry run, showing a list of all available models/configs/attaacks/mitigations and then
    /// showing what a full run would execute.
    #[arg(short, long)]
    dry: bool,

    /// Number of times to run each test. LLMs are non-deterministic, so a single run won't give
    /// all info. Defaults to 5.
    #[arg(short, long, default_value = "5")]
    number: i32,

    /// Remote LLMs tend to incur API costs, which means we exclude them by default.
    #[arg(short, long)]
    include_remote: bool,

    /// List of base config files, separated by commas pub_base1,priv_base2
    #[arg(short, long, value_parser=string_to_vec)]
    base_configs: Option<std::vec::Vec<String>>,

    /// List of config files, separated by commas: pub_conf1,priv_conf2
    #[arg(short='c', long, value_parser=string_to_vec)]
    additional_configs: Option<std::vec::Vec<String>>,

    /// List of attack files, separated by commas: pub_attack1,priv_attack2
    #[arg(short, long, value_parser=string_to_vec)]
    attacks: Option<std::vec::Vec<String>>,

    /// List of mitigation files. You can chain multiple by using '|': pub_first|priv_second,pub_other
    #[arg(short, long, value_parser=string_to_vec_vec)]
    mitigations: Option<std::vec::Vec<Vec<String>>>,
}

//TODO: Argument validation
pub fn string_to_vec(s: &str) -> Result<Vec<String>, String> {
    println!("strining to vecing");
    Ok(s.split(",").map(|x| x.to_string()).collect())
}

//TODO: Argument validation
pub fn string_to_vec_vec(v: &str) -> Result<Vec<String>, String> {
    Ok(v.split(",")
        .map(|s| s.split("|").map(|x| x.to_string()).collect())
        .collect())
}

#[tokio::main]
async fn main() {
    dotenv();
    Builder::new()
        .filter(None, LevelFilter::Info) // Default log level set to `info`
        .init();

    let cli = Cli::parse();

    println!("huh");

    let load_pub = !cli.no_public;
    let n = cli.number;
    let raw_id = std::env::var("PANTRY_CLIENT_ID").expect("PANTRY_CLIENT_ID must be set.");
    let pantry_id = Uuid::parse_str(&raw_id).unwrap();
    let pantry_key = std::env::var("PANTRY_CLIENT_KEY").expect("PANTRY_CLIENT_KEY must be set.");
    let pantry_client = pantry_rs::PantryClient::login(pantry_id, pantry_key, None);

    let loaded_configs = load::load_base_configs(load_pub, cli.base_configs, pantry_client.clone())
        .await
        .unwrap();
    info!("Loaded configs: {:#?}", loaded_configs);
    let additional_configs =
        load::load_additional_configs(load_pub, cli.additional_configs).unwrap();
    info!("Addtional configs: {:#?}", additional_configs);
    let attacks = load::load_attacks(load_pub, cli.attacks).unwrap();
    info!("Attacks: {:#?}", attacks);
    let mitigations = load::load_mitigations(load_pub, cli.mitigations)
        .await
        .unwrap();
    info!("Mitigation Sets: {:#?}", mitigations);

    let compiled_runs =
        compile::compile_runs(loaded_configs, additional_configs, attacks, mitigations).unwrap();
    info!("Compiled runs: {:#?}", compiled_runs);

    let results = execute::execute_full_runs(pantry_client, compiled_runs, n)
        .await
        .unwrap();

    info!("{}", "=====RESULTS=====".bold().cyan());
    for res in results.iter() {
        info!("\n{}", structures::RunSet(res.clone()));
    }
    // clap
    //
    //params:
    //--no-pub (don't test public library)
    //--serve (activate webserver to run prompt tests)
    // serve runs an attack prompt against every vulnerability given to the serve function
    // and every mitigation combination given to the serve function.
    // on every LLM given to the serve function.
    //  (maye we should give it full configs instead and just swap out the attack phrase?)
    //  4 llms, 1 additional config (no additional config!) 0 mitigation all mitigation
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
