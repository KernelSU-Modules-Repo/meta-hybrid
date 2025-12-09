use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::{conf::config, defs, core::inventory::Module};

#[derive(Debug)]
pub struct OverlayOperation {
    pub partition_name: String,
    pub target: String,
    pub lowerdirs: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct HymoOperation {
    pub module_id: String,
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Default)]
pub struct MountPlan {
    pub overlay_ops: Vec<OverlayOperation>,
    pub hymo_ops: Vec<HymoOperation>,
    pub magic_module_paths: Vec<PathBuf>,
    
    pub overlay_module_ids: Vec<String>,
    pub hymo_module_ids: Vec<String>,
    pub magic_module_ids: Vec<String>,
}

impl MountPlan {
    pub fn print_visuals(&self) {
        if self.overlay_ops.is_empty() && self.magic_module_paths.is_empty() && self.hymo_ops.is_empty() {
            log::info!(">> Empty plan. Standby mode.");
            return;
        }

        if !self.hymo_ops.is_empty() {
            log::info!("[HymoFS Injection Protocol]");
            let mut shown_modules = HashSet::new();
            for op in &self.hymo_ops {
                if shown_modules.insert(&op.module_id) {
                    log::info!("├── [Inject] {}", op.module_id);
                }
            }
        }

        if !self.overlay_ops.is_empty() {
            log::info!("[OverlayFS Fusion Sequence]");
            for (i, op) in self.overlay_ops.iter().enumerate() {
                let is_last_op = i == self.overlay_ops.len() - 1 && self.magic_module_paths.is_empty();
                let branch = if is_last_op { "╰──" } else { "├──" };
                
                log::info!("{} [Target: {}] {}", branch, op.partition_name, op.target);
                
                let prefix = if is_last_op { "    " } else { "│   " };

                for (j, layer) in op.lowerdirs.iter().enumerate() {
                    let is_last_layer = j == op.lowerdirs.len() - 1;
                    let sub_branch = if is_last_layer { "╰──" } else { "├──" };
                    
                    let mod_name = layer.parent()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_else(|| "UNKNOWN".into());
                        
                    log::info!("{}{} [Layer] {}", prefix, sub_branch, mod_name);
                }
            }
        }

        if !self.magic_module_paths.is_empty() {
            log::info!("[Magic Mount Fallback Protocol]");
            for (i, path) in self.magic_module_paths.iter().enumerate() {
                let is_last = i == self.magic_module_paths.len() - 1;
                let branch = if is_last { "╰──" } else { "├──" };
                let mod_name = path.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_else(|| "UNKNOWN".into());
                log::info!("{} [Bind] {}", branch, mod_name);
            }
        }
    }
}

pub fn generate(
    config: &config::Config, 
    modules: &[Module], 
    storage_root: &Path
) -> Result<MountPlan> {
    let mut plan = MountPlan::default();
    
    let mut partition_layers: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut magic_paths = HashSet::new();
    let mut overlay_ids = HashSet::new();
    let mut magic_ids = HashSet::new();
    let mut hymo_ids = HashSet::new();

    let mut target_partitions = defs::BUILTIN_PARTITIONS.to_vec();
    target_partitions.extend(config.partitions.iter().map(|s| s.as_str()));

    for module in modules {
        let mut content_path = storage_root.join(&module.id);
        
        if module.mode == "magic" {
            content_path = module.source_path.clone();
            if has_meaningful_content(&content_path, &target_partitions) {
                magic_paths.insert(content_path);
                magic_ids.insert(module.id.clone());
            }
            continue;
        } 
        
        if module.mode == "hymofs" {
            if !content_path.exists() {
                content_path = module.source_path.clone();
            }

            if content_path.exists() {
                let mut has_hymo_content = false;
                for part in &target_partitions {
                    let part_src = content_path.join(part);
                    let part_target = Path::new("/").join(part);
                    
                    if part_src.exists() && has_files(&part_src) {
                        plan.hymo_ops.push(HymoOperation {
                            module_id: module.id.clone(),
                            source: part_src,
                            target: part_target,
                        });
                        has_hymo_content = true;
                    }
                }
                if has_hymo_content {
                    hymo_ids.insert(module.id.clone());
                }
            }
            continue;
        }

        // OverlayFS (Default/Auto)
        if !content_path.exists() {
            log::debug!("Planner: Module {} content missing in storage, skipping", module.id);
            continue;
        }

        let mut participates_in_overlay = false;

        for part in &target_partitions {
            let part_path = content_path.join(part);
            
            if part_path.is_dir() && has_files(&part_path) {
                partition_layers.entry(part.to_string())
                    .or_default()
                    .push(part_path);
                participates_in_overlay = true;
            }
        }

        if participates_in_overlay {
            overlay_ids.insert(module.id.clone());
        }
    }

    for (part, layers) in partition_layers {
        let initial_target_path = format!("/{}", part);
        let target_path_obj = Path::new(&initial_target_path);
        let resolved_target = if target_path_obj.is_symlink() || target_path_obj.exists() {
            match target_path_obj.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("Planner: Failed to resolve path {}: {}. Skipping.", initial_target_path, e);
                    continue;
                }
            }
        } else {
            continue;
        };

        if !resolved_target.is_dir() {
            log::warn!("Planner: Target {} is not a directory, skipping", resolved_target.display());
            continue;
        }

        plan.overlay_ops.push(OverlayOperation {
            partition_name: part,
            target: resolved_target.to_string_lossy().to_string(),
            lowerdirs: layers,
        });
    }

    plan.magic_module_paths = magic_paths.into_iter().collect();
    plan.overlay_module_ids = overlay_ids.into_iter().collect();
    plan.magic_module_ids = magic_ids.into_iter().collect();
    plan.hymo_module_ids = hymo_ids.into_iter().collect();

    plan.overlay_module_ids.sort();
    plan.magic_module_ids.sort();
    plan.hymo_module_ids.sort();

    Ok(plan)
}

fn has_files(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for _ in entries.flatten() {
            return true;
        }
    }
    false
}

fn has_meaningful_content(base: &Path, partitions: &[&str]) -> bool {
    for part in partitions {
        let p = base.join(part);
        if p.exists() && has_files(&p) {
            return true;
        }
    }
    false
}
