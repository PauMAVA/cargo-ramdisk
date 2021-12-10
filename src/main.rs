extern crate nanoid;
extern crate structopt;
extern crate sys_mount;
#[macro_use]
extern crate carlog;

use carlog::prelude::*;

mod config;

use carlog::{carlog_ok, carlog_warning};
use config::{CargoRamdiskConfig, MountConfig, RemountConfig, Subcommands, UnmountConfig};
use nanoid::nanoid;
use std::env::current_dir;
use std::fs::{create_dir, read_link, remove_dir_all};
use std::io::Result;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

const BASE_RAMDISK_FOLDER: &str = "/dev/shm";

fn main() -> Result<()> {
    let config = CargoRamdiskConfig::from_args();
    if let Some(subcommand) = config.subcommand {
        match subcommand {
            Subcommands::Mount(config) => mount(config),
            Subcommands::Remount(config) => remount(config),
            Subcommands::Unmount(config) => unmount(config),
        }
    } else {
        mount(config.into())
    }
}

fn prepare_tmpfs_path(target: PathBuf) -> Result<(PathBuf, PathBuf, String, bool)> {
    let mut target_path = target;
    if target_path.is_relative() {
        target_path = current_dir()?.join(target_path);
    }
    carlog_ok!(
        "Preprocessed",
        format!("📁 The target path is {:?}", &target_path)
    );
    let shm_path_id = nanoid!();
    carlog_ok!(
        "Preprocessed",
        format!("💳 Generated unique project id {:?}", &shm_path_id)
    );
    let shm_path = PathBuf::from(format!("{}/target{}", BASE_RAMDISK_FOLDER, &shm_path_id));
    if let Ok(real_path) = read_link(target_path.clone()) {
        if real_path.starts_with(BASE_RAMDISK_FOLDER) {
            carlog_ok!("Preprocessed", "⛓ The target path is already a symlink to a tmpfs folder. Changing target tmpfs path...");
            return Ok((real_path, target_path, shm_path_id, true));
        } else {
            carlog_warning!("The target path was linked to a non-tmpfs filesystem...");
        }
    }
    carlog_ok!(
        "Preprocessed",
        format!("📁 The tmpfs path is {:?}", &shm_path)
    );
    if !shm_path.exists() {
        create_dir(shm_path.clone())?;
        carlog_ok!("Created", format!("{:?}", &shm_path));
        if target_path.exists() {
            remove_dir_all(target_path.clone())?;
            carlog_ok!("Deleted", format!("🗑 {:?}", &target_path));
        }
    }
    Ok((shm_path.clone(), target_path.clone(), shm_path_id, false))
}

pub fn mount(config: MountConfig) -> Result<()> {
    carlog_info!("Mounting", format!("Trying to mount {:?}", &config.target));
    let target = normalize_path(config.target);
    let (shm, target, _, linked) = prepare_tmpfs_path(target)?;
    if !linked {
        symlink(&shm, &target)?;
        carlog_ok!("Linked", format!("⛓ {:?} -> {:?}", shm, target));
    }
    carlog_ok!(
        "Success",
        format!("✅ Successfully created tmpfs ramdisk at {:?}", target)
    );
    Ok(())
}

pub fn remount(config: RemountConfig) -> Result<()> {
    carlog_info!(
        "Remounting",
        format!("Trying to remount {:?}", &config.target)
    );
    unmount(UnmountConfig::from(&config))?;
    carlog_ok!("Unmounted", format!("{:?}", &config.target));
    mount(MountConfig::from(&config))?;
    carlog_ok!("Mounted", format!("{:?}", &config.target));
    Ok(())
}

pub fn unmount(config: UnmountConfig) -> Result<()> {
    carlog_info!(
        "Unmounting",
        format!("Trying to unmount {:?}", &config.target)
    );
    let mut target = normalize_path(config.target);
    if target.is_relative() {
        target = current_dir()?.join(target);
    }
    if !target.exists() {
        carlog_error!("Cannot unmount non existing path!");
        exit(1);
    }
    if let Ok(link) = read_link(&target) {
        if link.starts_with(BASE_RAMDISK_FOLDER) {
            remove_dir_all(&target)?;
            carlog_ok!("Unlinked", format!("{:?} ⛔ {:?}", &link, &target));
            remove_dir_all(&link)?;
            carlog_ok!("Deleted", format!("🗑 {:?}", &link));
            carlog_ok!(
                "Success",
                format!("✅ Successfully unmounted ramdisk at {:?}", target)
            );
        } else {
            carlog_error!("The specified target path is not a ramdisk!");
        }
    } else {
        carlog_error!("The specified target path is not a symlink!");
        exit(1);
    }
    Ok(())
}

fn normalize_path(path: PathBuf) -> PathBuf {
    if let Some(path_str) = path.clone().to_str() {
        let mut chars = path_str.chars();
        if let Some(c) = chars.next_back() {
            if c == '/' {
                return PathBuf::from(chars.as_str());
            }
        }
        return path;
    } else {
        carlog_error!("Cannot normalize invalid UTF-8 path!");
        exit(1);
    }
}

#[cfg(test)]
mod test {
    use crate::{
        mount, remount, unmount, MountConfig, RemountConfig, UnmountConfig, BASE_RAMDISK_FOLDER,
    };
    use std::env::temp_dir;
    use std::fs::remove_dir_all;

    #[test]
    fn test_tmpfs() {
        let target = temp_dir().join("target");
        if target.exists() {
            remove_dir_all(target.clone()).expect("Failed to delete previous test target dir...");
        }
        mount(MountConfig {
            target: target.clone(),
        })
        .expect("Failed to mount test tmpfs...");
        assert!(target.exists());
        let link = target.read_link();
        assert!(link.is_ok());
        let link = link.unwrap();
        assert!(link.starts_with(BASE_RAMDISK_FOLDER));
        remount(RemountConfig {
            target: target.clone(),
        })
        .expect("Failed to remount test tmpfs");
        assert!(target.exists());
        let link = target.read_link();
        assert!(link.is_ok());
        let link = link.unwrap();
        assert!(link.starts_with(BASE_RAMDISK_FOLDER));
        unmount(UnmountConfig {
            target: target.clone(),
        })
        .expect("Failed to unmount test tmpfs");
        assert!(!target.exists());
    }
}
