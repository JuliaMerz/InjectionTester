use std::fs::{self, DirEntry};
use std::io::Write;
use std::path::Path;

fn visit_dirs(
    dir: &Path,
    cb: &dyn Fn(&DirEntry) -> (String, String),
) -> std::io::Result<Vec<(String, String)>> {
    println!("Running for: {:?}", dir);
    let mut entries = Vec::new();
    if dir.is_dir() {
        println!("looking in dir");
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() && path.file_name().unwrap() != "mod.rs" {
                entries.push(cb(&entry));
            }
        }
    }
    Ok(entries)
}

fn generate_for_type(directory_name: &str, config_type: &str) -> std::io::Result<()> {
    let build_path = format!("src/{}", directory_name);
    let out_dir = Path::new(&build_path);
    let mod_path = out_dir.join("mod.rs");
    let pub_mod_path = out_dir.join("public/mod.rs");
    let priv_mod_path = out_dir.join("private/mod.rs");
    println!("trying: {:?}", mod_path);
    let mut f = fs::File::create(&mod_path)?;
    let mut f_pub = fs::File::create(&pub_mod_path)?;
    let mut f_priv = fs::File::create(&priv_mod_path)?;

    writeln!(f, "// This file is auto-generated. Just add a config file.").unwrap();
    writeln!(f, "use std::collections::HashMap;").unwrap();
    writeln!(f, "use crate::framework::structures::{};", config_type).unwrap(); // Adjust based on actual type names
    if directory_name == "attacks" {
        writeln!(f, "mod vulnerabilities;").unwrap(); // Adjust based on actual type names
    }

    writeln!(f, "mod public;").unwrap(); // Adjust based on actual type names
    writeln!(f, "mod private;").unwrap(); // Adjust based on actual type names

    // Auto-generate mod declarations and build HashMap
    let mut pub_config_inserts = Vec::new();
    let mut priv_config_inserts = Vec::new();

    // Auto-generate mod declarations and build HashMap
    let public_entries = visit_dirs(
        &Path::new(&format!("src/{}/public/", directory_name)),
        &|entry| {
            println!("In dir: {:?}", entry);
            let config_raw = entry.file_name().into_string().unwrap();
            let config_name = config_raw.strip_suffix(".rs").unwrap().clone();
            (
                format!("pub mod {};", config_name),
                format!(
                    "map.insert(\"pub_\".to_string()+\"{}\", public::{}::config())",
                    config_name, config_name
                ),
            )
        },
    )?;

    let private_entries = visit_dirs(
        &Path::new(&format!("src/{}/private/", directory_name)),
        &|entry| {
            let config_name = entry.file_name().into_string().unwrap();
            (
                format!("pub mod {};", config_name),
                format!(
                    "map.insert(\"priv_\".to_string()+\"{}\", private::{}::config())",
                    config_name, config_name
                ),
            )
        },
    )?;

    for (mod_decl, insert) in public_entries.iter() {
        writeln!(f_pub, "{}", mod_decl).unwrap();
        pub_config_inserts.push(insert.clone());
    }

    for (mod_decl, insert) in private_entries.iter() {
        writeln!(f_priv, "{}", mod_decl).unwrap();
        priv_config_inserts.push(insert.clone());
    }

    // Generate the getter function
    writeln!(f, "").unwrap();
    writeln!(
        f,
        "pub fn get_pub_{}() -> HashMap<String, Vec<{}>> {{",
        directory_name, config_type,
    )
    .unwrap();
    writeln!(f, "    let mut map = HashMap::new();").unwrap();
    for insert in pub_config_inserts {
        writeln!(f, "    {};", insert).unwrap();
    }
    writeln!(f, "    map").unwrap();
    writeln!(f, "}}").unwrap();

    // Generate the getter function
    writeln!(f, "").unwrap();
    writeln!(
        f,
        "pub fn get_priv_{}() -> HashMap<String, Vec<{}>> {{",
        directory_name, config_type,
    )
    .unwrap();
    writeln!(f, "    let mut map = HashMap::new();").unwrap();
    for insert in priv_config_inserts {
        writeln!(f, "    {};", insert).unwrap();
    }
    writeln!(f, "    map").unwrap();
    writeln!(f, "}}").unwrap();

    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("Running build script");
    generate_for_type("base_configs", "BaseConfig")?;
    generate_for_type("additional_configs", "AdditionalConfig")?;
    generate_for_type("attacks", "Attack")?;
    generate_for_type("mitigations", "Mitigation")?;

    Ok(())
}
