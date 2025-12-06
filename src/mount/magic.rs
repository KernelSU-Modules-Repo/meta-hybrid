use std::{
    fs::{self, DirEntry, create_dir, create_dir_all, read_dir, read_link},
    os::unix::fs::{MetadataExt, symlink},
    path::{Path, PathBuf},
    collections::hash_map::Entry,
};

use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use rustix::{
    fs::{Gid, Mode, Uid, chmod, chown},
    mount::{
        MountFlags, MountPropagationFlags, UnmountFlags, mount, mount_bind, mount_change,
        mount_move, mount_remount, unmount,
    },
};

use crate::{
    defs::{DISABLE_FILE_NAME, REMOVE_FILE_NAME, SKIP_MOUNT_FILE_NAME},
    mount::{
        node::{Node, NodeFileType},
        try_umount::send_unmountable,
    },
    utils::{ensure_dir_exists, lgetfilecon, lsetfilecon},
};

const ROOT_PARTITIONS: [&str; 4] = [
    "vendor",
    "system_ext",
    "product",
    "odm",
];

fn merge_nodes(high: &mut Node, low: Node) {
    if high.module_path.is_none() {
        high.module_path = low.module_path;
        high.file_type = low.file_type;
        high.replace = low.replace;
    }

    for (name, low_child) in low.children {
        match high.children.entry(name) {
            Entry::Vacant(v) => {
                v.insert(low_child);
            }
            Entry::Occupied(mut o) => {
                merge_nodes(o.get_mut(), low_child);
            }
        }
    }
}

fn process_module(path: &Path, extra_partitions: &[String]) -> Result<(Node, Node)> {
    let mut root = Node::new_root("");
    let mut system = Node::new_root("system");

    if path.join(DISABLE_FILE_NAME).exists()
        || path.join(REMOVE_FILE_NAME).exists()
        || path.join(SKIP_MOUNT_FILE_NAME).exists()
    {
        return Ok((root, system));
    }

    let mod_system = path.join("system");
    if mod_system.is_dir() {
        system.collect_module_files(&mod_system)?;
    }

    for partition in ROOT_PARTITIONS {
        let mod_part = path.join(partition);
        if mod_part.is_dir() {
            let node = system.children.entry(partition.to_string())
                .or_insert_with(|| Node::new_root(partition));
            
            if node.file_type == NodeFileType::Symlink {
                node.file_type = NodeFileType::Directory;
                node.module_path = None;
            }

            node.collect_module_files(&mod_part)?;
        }
    }

    for partition in extra_partitions {
        if ROOT_PARTITIONS.contains(&partition.as_str()) || partition == "system" {
            continue;
        }

        let path_of_root = Path::new("/").join(partition);
        let path_of_system = Path::new("/system").join(partition);

        if path_of_root.is_dir() && path_of_system.is_symlink() {
            let name = partition.clone();
            let mod_part = path.join(partition);
            
            if mod_part.is_dir() {
                let node = root.children.entry(name)
                    .or_insert_with(|| Node::new_root(partition));
                node.collect_module_files(&mod_part)?;
            }
        } else if path_of_root.is_dir() {
            let name = partition.clone();
            let mod_part = path.join(partition);
            if mod_part.is_dir() {
                let node = root.children.entry(name)
                    .or_insert_with(|| Node::new_root(partition));
                node.collect_module_files(&mod_part)?;
            }
        }
    }

    Ok((root, system))
}

fn collect_module_files(module_paths: &[PathBuf], extra_partitions: &[String]) -> Result<Option<Node>> {
    let (mut final_root, mut final_system) = module_paths.par_iter()
        .map(|path| process_module(path, extra_partitions))
        .reduce(
            || Ok((Node::new_root(""), Node::new_root("system"))),
            |a, b| {
                let (mut r_a, mut s_a) = a?;
                let (r_b, s_b) = b?;
                merge_nodes(&mut r_a, r_b);
                merge_nodes(&mut s_a, s_b);
                Ok((r_a, s_a))
            }
        )?;

    let has_content = !final_root.children.is_empty() || !final_system.children.is_empty();

    if has_content {
        const BUILTIN_CHECKS: [(&str, bool); 4] = [
            ("vendor", true),
            ("system_ext", true),
            ("product", true),
            ("odm", false),
        ];

        for (partition, require_symlink) in BUILTIN_CHECKS {
            let path_of_root = Path::new("/").join(partition);
            let path_of_system = Path::new("/system").join(partition);

            if path_of_root.is_dir() && (!require_symlink || path_of_system.is_symlink()) {
                let name = partition.to_string();
                if let Some(node) = final_system.children.remove(&name) {
                    final_root.children.insert(name, node);
                }
            }
        }

        final_root.children.insert("system".to_string(), final_system);
        Ok(Some(final_root))
    } else {
        Ok(None)
    }
}

fn clone_symlink<S>(src: S, dst: S) -> Result<()>
where
    S: AsRef<Path>,
{
    let src_symlink = read_link(src.as_ref())?;
    symlink(&src_symlink, dst.as_ref())?;
    lsetfilecon(dst.as_ref(), lgetfilecon(src.as_ref())?.as_str())?;
    Ok(())
}

fn mount_mirror<P>(path: P, work_dir_path: P, entry: &DirEntry) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().join(entry.file_name());
    let work_dir_path = work_dir_path.as_ref().join(entry.file_name());
    let file_type = entry.file_type()?;

    if file_type.is_file() {
        fs::File::create(&work_dir_path)?;
        mount_bind(&path, &work_dir_path)?;
    } else if file_type.is_dir() {
        create_dir(&work_dir_path)?;
        let metadata = entry.metadata()?;
        chmod(&work_dir_path, Mode::from_raw_mode(metadata.mode()))?;
        chown(
            &work_dir_path,
            Some(Uid::from_raw(metadata.uid())),
            Some(Gid::from_raw(metadata.gid())),
        )?;
        
        lsetfilecon(&work_dir_path, lgetfilecon(&path)?.as_str())?;
        for entry in read_dir(&path)?.flatten() {
            mount_mirror(&path, &work_dir_path, &entry)?;
        }
    } else if file_type.is_symlink() {
        clone_symlink(&path, &work_dir_path)?;
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn do_magic_mount<P>(path: P, work_dir_path: P, current: Node, has_tmpfs: bool, disable_umount: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut current = current;
    let path = path.as_ref().join(&current.name);
    let work_dir_path = work_dir_path.as_ref().join(&current.name);
    match current.file_type {
        NodeFileType::RegularFile => {
            let target_path = if has_tmpfs {
                fs::File::create(&work_dir_path)?;
                &work_dir_path
            } else {
                &path
            };
            if let Some(module_path) = &current.module_path {
                mount_bind(module_path, target_path).with_context(|| {
                    if !disable_umount {
                        let _ = send_unmountable(target_path);
                    }
                    format!(
                        "mount module file {} -> {}",
                        module_path.display(),
                        work_dir_path.display(),
                    )
                })?;
                
                let _ = mount_change(target_path, MountPropagationFlags::PRIVATE);

                if let Err(e) =
                    mount_remount(target_path, MountFlags::RDONLY | MountFlags::BIND, "")
                {
                    log::warn!("make file {} ro: {e:#?}", target_path.display());
                }
            } else {
                bail!("cannot mount root file {}!", path.display());
            }
        }
        NodeFileType::Symlink => {
            if let Some(module_path) = &current.module_path {
                clone_symlink(module_path, &work_dir_path).with_context(|| {
                    format!(
                        "create module symlink {} -> {}",
                        module_path.display(),
                        work_dir_path.display(),
                    )
                })?;
            } else {
                bail!("cannot mount root symlink {}!", path.display());
            }
        }
        NodeFileType::Directory => {
            let mut create_tmpfs = !has_tmpfs && current.replace && current.module_path.is_some();
            if !has_tmpfs && !create_tmpfs {
                for it in &mut current.children {
                    let (name, node) = it;
                    let real_path = path.join(name);
                    let need = match node.file_type {
                        NodeFileType::Symlink => true,
                        NodeFileType::Whiteout => real_path.exists(),
                        _ => {
                            if let Ok(metadata) = real_path.symlink_metadata() {
                                let file_type = NodeFileType::from_file_type(metadata.file_type())
                                    .unwrap_or(NodeFileType::Whiteout);
                                file_type != node.file_type || file_type == NodeFileType::Symlink
                            } else {
                                true
                            }
                        }
                    };
                    if need {
                        if current.module_path.is_none() && !path.exists() {
                            log::error!(
                                "cannot create tmpfs on {}, ignore: {name}",
                                path.display()
                            );
                            node.skip = true;
                            continue;
                        }
                        create_tmpfs = true;
                        break;
                    }
                }
            }

            let has_tmpfs = has_tmpfs || create_tmpfs;

            if has_tmpfs {
                create_dir_all(&work_dir_path)?;
                let (metadata, path) = if path.exists() {
                    (path.metadata()?, &path)
                } else if let Some(module_path) = &current.module_path {
                    (module_path.metadata()?, module_path)
                } else {
                    bail!("cannot mount root dir {}!", path.display());
                };
                chmod(&work_dir_path, Mode::from_raw_mode(metadata.mode()))?;
                chown(
                    &work_dir_path,
                    Some(Uid::from_raw(metadata.uid())),
                    Some(Gid::from_raw(metadata.gid())),
                )?;
                
                lsetfilecon(&work_dir_path, lgetfilecon(path)?.as_str())?;
            }

            if create_tmpfs {
                mount_bind(&work_dir_path, &work_dir_path)
                    .context("bind self")
                    .with_context(|| {
                        format!(
                            "creating tmpfs for {} at {}",
                            path.display(),
                            work_dir_path.display(),
                        )
                    })?;
            }

            if path.exists() && !current.replace {
                for entry in path.read_dir()?.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let result = if let Some(node) = current.children.remove(&name) {
                        if node.skip {
                            continue;
                        }
                        do_magic_mount(&path, &work_dir_path, node, has_tmpfs, disable_umount)
                            .with_context(|| format!("magic mount {}/{name}", path.display()))
                    } else if has_tmpfs {
                        mount_mirror(&path, &work_dir_path, &entry)
                            .with_context(|| format!("mount mirror {}/{name}", path.display()))
                    } else {
                        Ok(())
                    };

                    if let Err(e) = result {
                        if has_tmpfs {
                            return Err(e);
                        }
                        log::error!("mount child {}/{name} failed: {e:#?}", path.display());
                    }
                }
            }

            if current.replace {
                if current.module_path.is_none() {
                    bail!(
                        "dir {} is declared as replaced but it is root!",
                        path.display()
                    );
                }
            }

            for (name, node) in current.children {
                if node.skip {
                    continue;
                }
                if let Err(e) = do_magic_mount(&path, &work_dir_path, node, has_tmpfs, disable_umount)
                    .with_context(|| format!("magic mount {}/{name}", path.display()))
                {
                    if has_tmpfs {
                        return Err(e);
                    }
                    log::error!("mount child {}/{name} failed: {e:#?}", path.display());
                }
            }

            if create_tmpfs {
                if let Err(e) =
                    mount_remount(&work_dir_path, MountFlags::RDONLY | MountFlags::BIND, "")
                {
                    log::warn!("make dir {} ro: {e:#?}", path.display());
                }
                mount_move(&work_dir_path, &path)
                    .context("move self")
                    .with_context(|| {
                        format!(
                            "moving tmpfs {} -> {}",
                            work_dir_path.display(),
                            path.display()
                        )
                    })?;
                if let Err(e) = mount_change(&path, MountPropagationFlags::PRIVATE) {
                    log::warn!("make dir {} private: {e:#?}", path.display());
                }
                if !disable_umount {
                    let _ = send_unmountable(path);
                }
            }
        }
        NodeFileType::Whiteout => {
            log::debug!("file {} is removed", path.display());
        }
    }

    Ok(())
}

pub fn mount_partitions(
    tmp_path: &Path,
    module_paths: &[PathBuf],
    mount_source: &str,
    extra_partitions: &[String],
    disable_umount: bool,
) -> Result<()> {
    if let Some(root) = collect_module_files(module_paths, extra_partitions)? {
        log::debug!("[Magic Mount Tree Constructed]");
        let tree_str = format!("{:?}", root);
        for line in tree_str.lines() {
            log::debug!("   {}", line);
        }

        let tmp_dir = tmp_path.join("workdir");
        ensure_dir_exists(&tmp_dir)?;
        mount(mount_source, &tmp_dir, "tmpfs", MountFlags::empty(), None::<&std::ffi::CStr>).context("mount tmp")?;
        
        mount_change(&tmp_dir, MountPropagationFlags::PRIVATE).context("make tmp private")?;

        let result = do_magic_mount(Path::new("/"), tmp_dir.as_path(), root, false, disable_umount);

        if let Err(e) = unmount(&tmp_dir, UnmountFlags::DETACH) {
            log::error!("failed to unmount tmp {e}");
        }
        fs::remove_dir(tmp_dir).ok();

        result
    } else {
        Ok(())
    }
}
