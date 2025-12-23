use std::ffi::CString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use log::{debug, info, warn};
use procfs::process::Process;
use rustix::{
    fd::AsFd,
    fs::CWD,
    mount::{
        MountFlags, MoveMountFlags, OpenTreeFlags, UnmountFlags, mount, move_mount, open_tree,
        unmount,
    },
};

use crate::defs::KSU_OVERLAY_SOURCE;

fn mount_overlayfs(
    lower_dirs: &[String],
    lowest: &str,
    upperdir: Option<&Path>,
    workdir: Option<&Path>,
    dest: &Path,
) -> Result<()> {
    let lowerdir_config = lower_dirs
        .iter()
        .map(|s| s.as_str())
        .chain(std::iter::once(lowest))
        .collect::<Vec<_>>()
        .join(":");

    debug!(
        "mount overlayfs on {:?}, lowerdir={}, upperdir={:?}, workdir={:?}",
        dest, lowerdir_config, upperdir, workdir
    );

    let mut data = format!("lowerdir={}", lowerdir_config);
    if let (Some(u), Some(w)) = (upperdir, workdir) {
        data = format!("{},upperdir={},workdir={}", data, u.display(), w.display());
    }

    let data_c = CString::new(data).context("Failed to create CString for mount data")?;

    mount(
        KSU_OVERLAY_SOURCE,
        dest,
        "overlay",
        MountFlags::empty(),
        Some(&data_c),
    )
    .with_context(|| format!("Legacy mount failed for {}", dest.display()))?;

    Ok(())
}

pub fn bind_mount(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    debug!(
        "bind mount {} -> {}",
        from.as_ref().display(),
        to.as_ref().display()
    );
    let tree = open_tree(
        CWD,
        from.as_ref(),
        OpenTreeFlags::OPEN_TREE_CLOEXEC
            | OpenTreeFlags::OPEN_TREE_CLONE
            | OpenTreeFlags::AT_RECURSIVE,
    )?;
    move_mount(
        tree.as_fd(),
        "",
        CWD,
        to.as_ref(),
        MoveMountFlags::MOVE_MOUNT_F_EMPTY_PATH,
    )?;
    Ok(())
}

fn mount_overlay_child(
    mount_point: &str,
    relative: &str,
    module_roots: &[String],
    stock_root: &str,
) -> Result<()> {
    if !module_roots
        .iter()
        .any(|lower| Path::new(&format!("{}{}", lower, relative)).exists())
    {
        return bind_mount(stock_root, mount_point);
    }

    if !Path::new(stock_root).is_dir() {
        return Ok(());
    }

    let mut lower_dirs = Vec::new();
    for lower in module_roots {
        let lower_dir = format!("{}{}", lower, relative);
        let path = Path::new(&lower_dir);
        if path.is_dir() {
            lower_dirs.push(lower_dir);
        } else if path.exists() {
            return Ok(());
        }
    }

    if lower_dirs.is_empty() {
        return Ok(());
    }

    if let Err(e) = mount_overlayfs(&lower_dirs, stock_root, None, None, Path::new(mount_point)) {
        warn!(
            "failed to mount overlay child {}: {:#}, fallback to bind",
            mount_point, e
        );
        bind_mount(stock_root, mount_point)?;
    }
    Ok(())
}

pub fn mount_overlay(
    target: &str,
    module_roots: &[String],
    workdir: Option<PathBuf>,
    upperdir: Option<PathBuf>,
    disable_umount: bool,
) -> Result<()> {
    info!("mount overlay for {}", target);

    std::env::set_current_dir(target).with_context(|| format!("failed to chdir to {}", target))?;
    let stock_root = ".";

    let mounts = Process::myself()?
        .mountinfo()
        .with_context(|| "get mountinfo")?;

    let mount_seq = mounts
        .0
        .iter()
        .filter(|m| {
            m.mount_point.starts_with(target) && !Path::new(target).starts_with(&m.mount_point)
        })
        .map(|m| m.mount_point.to_str());

    let mut valid_mount_seq: Vec<&str> = mount_seq.flatten().collect();
    valid_mount_seq.sort();
    valid_mount_seq.dedup();

    mount_overlayfs(
        module_roots,
        stock_root,
        upperdir.as_deref(),
        workdir.as_deref(),
        Path::new(target),
    )
    .with_context(|| "mount overlayfs for root failed")?;

    for mount_point in valid_mount_seq {
        let relative: String = mount_point.replacen(target, "", 1);
        let child_stock_root = format!("{}{}", stock_root, relative);

        if !Path::new(&child_stock_root).exists() {
            continue;
        }

        if let Err(e) = mount_overlay_child(mount_point, &relative, module_roots, &child_stock_root)
        {
            warn!(
                "failed to mount overlay for child {}: {:#}, revert",
                mount_point, e
            );
            if !disable_umount {
                let _ = unmount(target, UnmountFlags::empty());
            }
            bail!(e);
        }
    }

    Ok(())
}
