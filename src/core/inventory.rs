use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::{defs, conf::config, core::policy};

#[derive(Debug, Clone)]
pub struct Module {
    pub id: String,
    pub source_path: PathBuf,
    pub partitions: Vec<String>,
    pub mode: String,
}

pub fn scan(source_dir: &Path, _config: &config::Config, settings: &policy::ModuleSettings) -> Result<Vec<Module>> {
    let mut modules = Vec::new();
    if !source_dir.exists() {
        return Ok(modules);
    }

    let mut entries: Vec<_> = fs::read_dir(source_dir)?
        .filter_map(|e| e.ok())
        .collect();
    
    entries.sort_by_key(|e| e.file_name());
    entries.reverse();

    for entry in entries {
        let path = entry.path();
        
        if !path.is_dir() { continue; }
        
        let id = entry.file_name().to_string_lossy().to_string();
        
        if id == "meta-hybrid" || id == "lost+found" || id == ".git" { continue; }

        if path.join(defs::DISABLE_FILE_NAME).exists() || 
           path.join(defs::REMOVE_FILE_NAME).exists() || 
           path.join(defs::SKIP_MOUNT_FILE_NAME).exists() { 
            continue; 
        }
        let mut partitions = Vec::new();
        for &part_name in defs::BUILTIN_PARTITIONS {
            if path.join(part_name).is_dir() {
                partitions.push(part_name.to_string());
            }
        }

        let mode_enum = settings.get_mode(&id, None);
        let mode = mode_enum.to_string();

        if !partitions.is_empty() {
            modules.push(Module {
                id,
                source_path: path,
                partitions,
                mode,
            });
        }
    }

    Ok(modules)
}
