use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use crate::core::inventory::Module;
use crate::core::policy::{ModuleSettings, MountMode};

#[derive(Debug)]
pub struct MountPlan {
    pub overlay_targets: HashMap<String, Vec<PathBuf>>,
    pub magic_targets: Vec<(PathBuf, PathBuf)>,
    pub hymo_targets: Vec<(PathBuf, PathBuf)>,
}

pub fn generate(modules: &[Module], settings: &ModuleSettings) -> Result<MountPlan> {
    let mut overlay_targets: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut magic_targets = Vec::new();
    let mut hymo_targets = Vec::new();

    for module in modules {
        for partition in &module.partitions {
            let mode = settings.get_mode(&module.id, Some(partition));
            let source = module.source_path.join(partition);
            
            let is_dir = source.is_dir();

            match mode {
                MountMode::Magic => {
                    let target = PathBuf::from("/").join(partition);
                    magic_targets.push((source, target));
                }
                MountMode::Hymo => {
                    let target = PathBuf::from("/").join(partition);
                    hymo_targets.push((source, target));
                }
                _ => {
                    if is_dir {
                        overlay_targets
                            .entry(partition.to_string())
                            .or_default()
                            .push(source);
                    } else {
                        let target = PathBuf::from("/").join(partition);
                        magic_targets.push((source, target));
                    }
                }
            }
        }
    }

    magic_targets.reverse();
    hymo_targets.reverse();

    Ok(MountPlan {
        overlay_targets,
        magic_targets,
        hymo_targets,
    })
}
