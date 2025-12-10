use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::Result;
use serde::Serialize;
use crate::conf::config::Config;
use crate::core::inventory;

#[derive(Serialize)]
struct ModuleInfo {
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    mode: String,
    rules: inventory::ModuleRules,
}

pub fn print_list(config: &Config) -> Result<()> {
    let modules = inventory::scan(&config.moduledir, config)?;
    let mut infos = Vec::new();

    for m in modules {
        let prop_path = m.source_path.join("module.prop");
        let (name, version, author, description) = read_module_prop(&prop_path);
        let mode_str = match m.rules.default_mode {
            inventory::MountMode::Overlay => "auto",
            inventory::MountMode::HymoFs => "hymofs",
            inventory::MountMode::Magic => "magic",
            inventory::MountMode::Ignore => "ignore",
        };

        infos.push(ModuleInfo {
            id: m.id.clone(),
            name,
            version,
            author,
            description,
            mode: mode_str.to_string(),
            rules: m.rules,
        });
    }

    println!("{}", serde_json::to_string(&infos)?);
    Ok(())
}

fn read_module_prop(path: &Path) -> (String, String, String, String) {
    let mut name = String::new();
    let mut version = String::new();
    let mut author = String::new();
    let mut description = String::new();

    if let Ok(file) = fs::File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(l) = line {
                if let Some((k, v)) = l.split_once('=') {
                    let val = v.trim().to_string();
                    match k.trim() {
                        "name" => name = val,
                        "version" => version = val,
                        "author" => author = val,
                        "description" => description = val,
                        _ => {}
                    }
                }
            }
        }
    }
    (name, version, author, description)
}
