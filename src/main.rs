mod conf;
mod core;
mod defs;
mod mount;
mod utils;

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use clap::Parser;
use mimalloc::MiMalloc;

use conf::{
    cli::{Cli, Commands},
    config::{Config, CONFIG_FILE_DEFAULT},
};
use core::{
    executor,
    inventory,
    planner,
    state::RuntimeState,
    storage,
    sync,
    modules,
    policy,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn load_config(cli: &Cli) -> Result<Config> {
    if let Some(config_path) = &cli.config {
        return Config::from_file(config_path);
    }
    match Config::load_default() {
        Ok(config) => Ok(config),
        Err(e) => {
            if Path::new(CONFIG_FILE_DEFAULT).exists() {
                eprintln!("Error loading config: {:#}", e);
            }
            Ok(Config::default())
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = &cli.command {
        match command {
            Commands::GenConfig { output } => { 
                Config::default().save_to_file(output)?; 
                return Ok(()); 
            },
            Commands::ShowConfig => { 
                let config = load_config(&cli)?;
                println!("{}", serde_json::to_string(&config)?); 
                return Ok(()); 
            },
            Commands::SaveConfig { payload } => {
                let json_bytes = (0..payload.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&payload[i..i + 2], 16))
                    .collect::<Result<Vec<u8>, _>>()
                    .context("Failed to decode hex payload")?;
                
                let config: Config = serde_json::from_slice(&json_bytes)
                    .context("Failed to parse config JSON")?;
                
                config.save_to_file(CONFIG_FILE_DEFAULT)?;
                println!("Configuration saved successfully.");
                return Ok(());
            },
            Commands::Storage => { 
                storage::print_status()?; 
                return Ok(()); 
            },
            Commands::Modules => { 
                let config = load_config(&cli)?;
                modules::print_list(&config)?; 
                return Ok(()); 
            }
        }
    }

    let mut config = load_config(&cli)?;
    config.merge_with_cli(
        cli.moduledir.clone(), 
        cli.tempdir.clone(), 
        cli.mountsource.clone(), 
        cli.verbose, 
        cli.partitions.clone(),
        cli.dry_run,
    );
    let settings = policy::ModuleSettings::load().unwrap_or_default();

    if config.dry_run {
        env_logger::builder()
            .filter_level(if config.verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
            .init();
        
        log::info!(":: DRY-RUN MODE ACTIVE ::");
        log::info!("No file system changes will be made.");

        let module_list = inventory::scan(&config.moduledir, &config)?;
        log::info!(">> Inventory Scan: Found {} enabled modules.", module_list.len());

        log::info!(">> simulating sync (skipped)");
        log::info!(">> simulating storage setup (skipped)");
        log::info!(">> Planning mount strategy using source files...");
        
        let plan = planner::generate(&module_list, &settings)?;
        
        log::info!(">> Overlay Targets: {:?}", plan.overlay_targets);
        log::info!(">> Magic Targets: {:?}", plan.magic_targets);
        log::info!(">> Hymo Targets: {:?}", plan.hymo_targets);
        
        log::info!(">> Dry-run complete. Exiting.");
        return Ok(());
    }

    let _log_guard = utils::init_logging(config.verbose, Path::new(defs::DAEMON_LOG_FILE))?;

    let camouflage_name = utils::random_kworker_name();
    if let Err(e) = utils::camouflage_process(&camouflage_name) {
        log::warn!("Failed to camouflage process: {}", e);
    }

    log::info!(">> Initializing Meta-Hybrid Mount Daemon...");
    log::debug!("Process camouflaged as: {}", camouflage_name);

    if config.disable_umount {
        log::warn!("!! Namespace Detach (try_umount) is DISABLED via config.");
    }

    utils::ensure_dir_exists(defs::RUN_DIR)?;

    let mnt_base = PathBuf::from(defs::FALLBACK_CONTENT_DIR);
    let img_path = Path::new(defs::BASE_DIR).join("modules.img");
    
    let storage_handle = storage::setup(&mnt_base, &img_path, config.force_ext4)?;
    log::info!(">> Storage Backend: [{}]", storage_handle.mode.to_uppercase());

    let module_list = inventory::scan(&config.moduledir, &config)?;
    log::info!(">> Inventory Scan: Found {} enabled modules.", module_list.len());

    sync::perform_sync(&module_list, &storage_handle.mount_point)?;

    let plan = planner::generate(&module_list, &settings)?;

    let active_mounts: Vec<String> = plan.overlay_targets.keys().cloned().collect();

    log::info!(">> Link Start! Executing mount plan...");
    let exec_result = executor::execute(&plan, &config)?;

    let mut nuke_active = false;
    if storage_handle.mode == "ext4" && config.enable_nuke {
        log::info!(">> Engaging Paw Pad Protocol (Stealth)...");
        match utils::ksu_nuke_sysfs(storage_handle.mount_point.to_string_lossy().as_ref()) {
            Ok(_) => {
                log::info!(">> Success: Paw Pad active. Sysfs traces purged.");
                nuke_active = true;
            },
            Err(e) => {
                log::warn!("!! Paw Pad failure: {}", e);
            }
        }
    }

    modules::update_description(
        &storage_handle.mode, 
        nuke_active, 
        exec_result.overlay_module_ids.len(), 
        exec_result.magic_module_ids.len(),
        exec_result.hymo_module_ids.len()
    );

    let storage_stats = storage::get_usage(&storage_handle.mount_point);
    let hymofs_available = storage::is_hymofs_active();

    let state = RuntimeState::new(
        storage_handle.mode,
        storage_handle.mount_point,
        exec_result.overlay_module_ids,
        exec_result.magic_module_ids,
        nuke_active,
        active_mounts,
        storage_stats,
        hymofs_available
    );
    
    if let Err(e) = state.save() {
        log::error!("Failed to save runtime state: {}", e);
    }

    log::info!(">> System operational. Mount sequence complete.");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        log::error!("!! Fatal Error: {:#}", e);
        eprintln!("Fatal Error: {:#}", e);
        std::process::exit(1);
    }
}
