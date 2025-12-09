use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use crate::defs::MountMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    #[serde(default)]
    pub default_mode: MountMode,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub partitions: HashMap<String, MountMode>,
}

impl Default for PartitionConfig {
    fn default() -> Self {
        Self {
            default_mode: MountMode::Auto,
            partitions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleSettings {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub modules: HashMap<String, PartitionConfig>,
}

impl ModuleSettings {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read settings file: {:?}", path))?;
        let settings = serde_json::from_str(&content)
            .with_context(|| "Failed to parse module settings JSON")?;
        Ok(settings)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write settings file: {:?}", path))?;
        Ok(())
    }

    pub fn get_mode(&self, module_id: &str, partition: Option<&str>) -> MountMode {
        if let Some(config) = self.modules.get(module_id) {
            if let Some(part) = partition {
                if let Some(mode) = config.partitions.get(part) {
                    return *mode;
                }
            }
            return config.default_mode;
        }
        MountMode::Auto
    }

    pub fn set_mode(&mut self, module_id: String, partition: Option<String>, mode: MountMode) {
        let config = self.modules.entry(module_id).or_default();
        match partition {
            Some(part) => {
                config.partitions.insert(part, mode);
            }
            None => {
                config.default_mode = mode;
            }
        }
    }
}
