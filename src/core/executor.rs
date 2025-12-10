use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
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
    pub magic_module_ids: Vec<String>,
    pub hymo_module_ids: Vec<String>,
}

fn extract_module_root(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        if current.join("module.prop").exists() {
            return Some(current.to_path_buf());
        }
        match current.parent() {
            Some(p) => current = p,
            None => break,
        }
        if current.to_string_lossy().len() < 10 { 
            break; 
        }
    }
    path.parent().map(|p| p.to_path_buf())
}

fn extract_id(path: &Path) -> Option<String> {
    extract_module_root(path)
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
}

struct OverlayOp {
    partition: String,
    target: String,
    lowerdirs: Vec<PathBuf>,
}

struct OverlayResult {
    magic_roots: Vec<PathBuf>,
    failed_ids: Vec<String>,
    success_records: Vec<(PathBuf, String)>,
}

pub fn execute(plan: &MountPlan, config: &config::Config) -> Result<ExecutionResult> {
    let mut magic_queue: Vec<PathBuf> = Vec::new();
    let mut global_success_map: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    
    let mut final_overlay_ids = HashSet::new();
    let mut final_magic_ids = Vec::new();
    let mut final_hymo_ids = HashSet::new();

    for (source, _) in &plan.magic_targets {
        if let Some(root) = extract_module_root(source) {
            magic_queue.push(root);
        }
    }

    let mut pending_hymo_fallbacks: Vec<(PathBuf, String)> = Vec::new();
    
    if !plan.hymo_targets.is_empty() {
        if HymoFs::is_available() {
            log::info!(">> Phase 1: HymoFS Injection...");
            if let Err(e) = HymoFs::clear() {
                log::warn!("Failed to reset HymoFS rules: {}", e);
            }

            for (source, target) in &plan.hymo_targets {
                let part_name = target.file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                let module_id = extract_id(source).unwrap_or_default();

                log::debug!("Injecting {} -> {}", source.display(), target.display());
                
                match HymoFs::inject_directory(target, source) {
                    Ok(_) => {
                        if let Some(root) = extract_module_root(source) {
                            global_success_map.entry(root).or_default().insert(part_name);
                        }
                        if !module_id.is_empty() {
                            final_hymo_ids.insert(module_id);
                        }
                    },
                    Err(e) => {
                        log::error!("HymoFS failed for {}: {}. Queueing for Overlay/Magic.", module_id, e);
                        
                        let relative_str = target.to_string_lossy();
                        let clean_partition = relative_str.trim_start_matches('/').to_string();

                        if source.is_dir() {
                            pending_hymo_fallbacks.push((source.clone(), clean_partition));
                        } else {
                            if let Some(root) = extract_module_root(source) {
                                magic_queue.push(root);
                            }
                        }
                    }
                }
            }
        } else {
            log::warn!("!! HymoFS requested but kernel support is missing. Falling back.");
            for (source, target) in &plan.hymo_targets {
                let relative_str = target.to_string_lossy();
                let clean_partition = relative_str.trim_start_matches('/').to_string();
                 
                if source.is_dir() {
                    pending_hymo_fallbacks.push((source.clone(), clean_partition));
                } else {
                    if let Some(root) = extract_module_root(source) {
                        magic_queue.push(root);
                    }
                }
            }
        }
    }

    let mut overlay_map: HashMap<String, Vec<PathBuf>> = plan.overlay_targets.clone();

    for (source, partition) in pending_hymo_fallbacks {
        overlay_map.entry(partition).or_default().push(source);
    }

    let mut overlay_ops: Vec<OverlayOp> = Vec::new();
    for (partition, lowerdirs) in &overlay_map {
        for src in lowerdirs {
            if let Some(id) = extract_id(src) {
                final_overlay_ids.insert(id);
            }
        }

        let target = format!("/{}", partition);
        if Path::new(&target).is_dir() {
             overlay_ops.push(OverlayOp {
                partition: partition.clone(),
                target,
                lowerdirs: lowerdirs.clone(),
            });
        } else {
             log::warn!("Skipping OverlayFS for '{}': Target is not a directory.", target);
             for src in lowerdirs {
                if let Some(root) = extract_module_root(src) {
                    magic_queue.push(root);
                }
             }
        }
    }

    log::info!(">> Phase 3: OverlayFS Execution...");
    let overlay_results: Vec<OverlayResult> = overlay_ops.par_iter()
        .map(|op| {
            let lowerdir_strings: Vec<String> = op.lowerdirs.iter()
                .map(|p| p.display().to_string())
                .collect();
                
            log::info!("Mounting {} [OVERLAY] ({} layers)", op.target, lowerdir_strings.len());
            
            if let Err(e) = overlay::mount_overlay(&op.target, &lowerdir_strings, None, None, config.disable_umount) {
                log::warn!("OverlayFS failed for {}: {}. Triggering fallback.", op.target, e);
                
                let mut local_magic = Vec::new();
                let mut local_failed_ids = Vec::new();

                for layer_path in &op.lowerdirs {
                    if let Some(root) = extract_module_root(layer_path) {
                        local_magic.push(root);
                        if let Some(id) = extract_id(layer_path) {
                            local_failed_ids.push(id);
                        }
                    }
                }
                return OverlayResult {
                    magic_roots: local_magic,
                    failed_ids: local_failed_ids,
                    success_records: Vec::new(),
                };
            }
            
            let mut successes = Vec::new();
            for layer_path in &op.lowerdirs {
                 if let Some(root) = extract_module_root(layer_path) {
                     successes.push((root, op.partition.clone()));
                 }
            }

            OverlayResult {
                magic_roots: Vec::new(),
                failed_ids: Vec::new(),
                success_records: successes,
            }
        })
        .collect();

    for res in overlay_results {
        magic_queue.extend(res.magic_roots);
        
        for id in res.failed_ids {
            final_overlay_ids.remove(&id); 
        }
        
        for (root, partition) in res.success_records {
            global_success_map.entry(root)
                .or_default()
                .insert(partition);
        }
    }

    magic_queue.sort();
    magic_queue.dedup();

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
        
        log::info!(">> Phase 4: Magic Mount (Complementary & Fallback) for {} modules...", magic_queue.len());
        
        utils::ensure_temp_dir(&tempdir)?;
        
        if let Err(e) = magic::mount_partitions(
            &tempdir, 
            &magic_queue, 
            &config.mountsource, 
            &config.partitions, 
            global_success_map, 
            config.disable_umount
        ) {
            log::error!("Magic Mount critical failure: {:#}", e);
            final_magic_ids.clear();
        }
        
        utils::cleanup_temp_dir(&tempdir);
    }

    let mut result_overlay = final_overlay_ids.into_iter().collect::<Vec<_>>();
    let mut result_magic = final_magic_ids;
    let mut result_hymo = final_hymo_ids.into_iter().collect::<Vec<_>>();

    result_overlay.sort();
    result_magic.sort();
    result_magic.dedup();
    result_hymo.sort();

    Ok(ExecutionResult {
        overlay_module_ids: result_overlay,
        magic_module_ids: result_magic,
        hymo_module_ids: result_hymo,
    })
}
