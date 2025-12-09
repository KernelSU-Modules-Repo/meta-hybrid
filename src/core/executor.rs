use std::path::{Path, PathBuf};
use anyhow::Result;
use rayon::prelude::*;
use crate::{
    conf::config, 
    mount::{magic, overlay, hymofs::HymoFs}, 
    utils,
    core::planner::MountPlan
};

pub struct ExecutionResult {
    pub overlay_module_ids: Vec<String>,
    pub hymo_module_ids: Vec<String>,
    pub magic_module_ids: Vec<String>,
}

fn extract_id(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
}

fn extract_module_root(partition_path: &Path) -> Option<PathBuf> {
    partition_path.parent().map(|p| p.to_path_buf())
}

pub fn execute(plan: &MountPlan, config: &config::Config) -> Result<ExecutionResult> {
    let mut magic_queue = plan.magic_module_paths.clone();
    let mut final_overlay_ids = plan.overlay_module_ids.clone();
    let mut final_hymo_ids = plan.hymo_module_ids.clone();
    
    // --- HymoFS Execution ---
    let mut hymo_fallback_ids = Vec::new();
    if !plan.hymo_ops.is_empty() {
        if HymoFs::is_available() {
            log::info!(">> Initializing HymoFS Engine...");
            if let Err(e) = HymoFs::clear() {
                log::warn!("Failed to reset HymoFS rules: {}", e);
            }

            for op in &plan.hymo_ops {
                log::debug!("Injecting {} -> {}", op.source.display(), op.target.display());
                if let Err(e) = HymoFs::inject_directory(&op.target, &op.source) {
                    log::error!("HymoFS injection failed for {}: {}", op.module_id, e);
                    // Mark for fallback
                    if let Some(root) = extract_module_root(&op.source) {
                        magic_queue.push(root);
                        hymo_fallback_ids.push(op.module_id.clone());
                    }
                }
            }
        } else {
            log::warn!("!! HymoFS requested but kernel support is missing. Falling back to Magic Mount.");
            for op in &plan.hymo_ops {
                 if let Some(root) = extract_module_root(&op.source) {
                    magic_queue.push(root);
                    hymo_fallback_ids.push(op.module_id.clone());
                }
            }
        }
    }
    
    if !hymo_fallback_ids.is_empty() {
        final_hymo_ids.retain(|id| !hymo_fallback_ids.contains(id));
    }

    // --- OverlayFS Execution ---
    let fallback_data: Vec<(Vec<PathBuf>, Vec<String>)> = plan.overlay_ops.par_iter()
        .map(|op| {
            let lowerdir_strings: Vec<String> = op.lowerdirs.iter()
                .map(|p| p.display().to_string())
                .collect();
                
            log::info!("Mounting {} [OVERLAY] ({} layers)", op.target, lowerdir_strings.len());
            
            if let Err(e) = overlay::mount_overlay(&op.target, &lowerdir_strings, None, None, config.disable_umount) {
                log::warn!("OverlayFS failed for {}: {}. Triggering fallback.", op.target, e);
                
                let mut local_magic = Vec::new();
                let mut local_fallback_ids = Vec::new();

                for layer_path in &op.lowerdirs {
                    if let Some(root) = extract_module_root(layer_path) {
                        local_magic.push(root.clone());
                        if let Some(id) = extract_id(layer_path) {
                            local_fallback_ids.push(id);
                        }
                    }
                }
                return (local_magic, local_fallback_ids);
            }
            
            (Vec::new(), Vec::new())
        })
        .collect();

    let mut fallback_ids = Vec::new();
    for (paths, ids) in fallback_data {
        magic_queue.extend(paths);
        fallback_ids.extend(ids);
    }

    if !fallback_ids.is_empty() {
        final_overlay_ids.retain(|id| !fallback_ids.contains(id));
        log::info!("{} modules fell back to Magic Mount.", fallback_ids.len());
    }

    // --- Magic Mount Execution ---
    magic_queue.sort();
    magic_queue.dedup();

    let mut final_magic_ids = Vec::new();

    if !magic_queue.is_empty() {
        let tempdir = if let Some(t) = &config.tempdir { 
            t.clone() 
        } else { 
            utils::select_temp_dir()? 
        };
        
        for path in &magic_queue {
            if let Some(name) = path.file_name() {
                final_magic_ids.push(name.to_string_lossy().to_string());
            }
        }
        
        log::info!("Executing Magic Mount for {} modules...", magic_queue.len());
        
        utils::ensure_temp_dir(&tempdir)?;
        
        if let Err(e) = magic::mount_partitions(
            &tempdir, 
            &magic_queue, 
            &config.mountsource, 
            &config.partitions, 
            config.disable_umount
        ) {
            log::error!("Magic Mount critical failure: {:#}", e);
            final_magic_ids.clear();
        }
        
        utils::cleanup_temp_dir(&tempdir);
    }

    final_overlay_ids.sort();
    final_overlay_ids.dedup();
    final_hymo_ids.sort();
    final_hymo_ids.dedup();
    final_magic_ids.sort();
    final_magic_ids.dedup();

    Ok(ExecutionResult {
        overlay_module_ids: final_overlay_ids,
        hymo_module_ids: final_hymo_ids,
        magic_module_ids: final_magic_ids,
    })
}
