use std::collections::HashMap;
use std::fs;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::defs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountMode {
    Auto,
    Overlay,
    Magic,
    Hymo,
}

impl Default for MountMode {
    fn default() -> Self {
        Self::Auto
    }
}

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

#[derive(Debug, Deserialize)]
pub struct ModuleDTO {
    pub id: String,
    pub config: PartitionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleSettings {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub modules: HashMap<String, PartitionConfig>,
}

impl ModuleSettings {
    pub fn load() -> Result<Self> {
        let path = std::path::Path::new(defs::MODULE_SETTINGS_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read settings file: {:?}", path))?;
        let settings = serde_json::from_str(&content)
            .with_context(|| "Failed to parse module settings JSON")?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let path = std::path::Path::new(defs::MODULE_SETTINGS_FILE);
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

    pub fn import_from_list(&mut self, list: Vec<ModuleDTO>) {
        for item in list {
            self.set_mode(item.id.clone(), None, item.config.default_mode);
            for (part, mode) in item.config.partitions {
                self.set_mode(item.id.clone(), Some(part), mode);
            }
        }
    }
}
