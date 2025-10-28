extern crate nanoid;
extern crate structopt;
#[macro_use]
extern crate carlog;

use carlog::prelude::*;

mod config;

use carlog::{carlog_ok, carlog_warning};
use config::{CargoRamdiskConfig, MountConfig, RemountConfig, Subcommands, UnmountConfig};
use nanoid::nanoid;
use std::env;
use std::env::current_dir;
use std::fs::{create_dir, read_link, remove_dir, remove_dir_all, remove_file};
use std::io::Result;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

const BASE_RAMDISK_FOLDER: &str = "/dev/shm";

fn main() -> Result<()> {
    let mut args = env::args().peekable();
    let _bin = args.next().expect("No program name...");
    if let Some(s) = args.peek() {
        if s.to_lowercase() == "ramdisk" {
            args.next().unwrap();
        }
    }
    let config = CargoRamdiskConfig::from_iter(args);
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

fn prepare_tmpfs_path(target: PathBuf, copy_to: bool) -> Result<(PathBuf, PathBuf, String, bool)> {
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
            if real_path.exists() {
                carlog_ok!("Preprocessed", "⛓ The target path is already a symlink to a tmpfs folder. Changing target tmpfs path...");
                return Ok((real_path, target_path, shm_path_id, true));
            } else {
                carlog_warning!(
                    "The target path was already a symlink to a non-existent tmpfs folder. Unlinking"
                );
                remove_file(target_path.clone())?;
            }
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
            if copy_to {
                carlog_info!(
                    "Copying",
                    format!("📁 {:?} -> {:?}", &target_path, &shm_path)
                );
                remove_dir(&shm_path)?;
                match std::process::Command::new("cp")
                    .arg("-r")
                    .arg("--preserve=mode,ownership,timestamps")
                    .arg(&target_path)
                    .arg(&shm_path)
                    .output()
                {
                    Ok(_) => {
                        carlog_ok!(
                            "Copied",
                            format!("📁 {:?} -> {:?}", &target_path, &shm_path)
                        );
                    }
                    Err(e) => {
                        carlog_error!(&format!(
                            "Failed to copy target path {:?} to tmpfs path {:?}. Error: {}",
                            &target_path, &shm_path, e
                        ));
                        remove_dir_all(&shm_path)?;
                        exit(1);
                    }
                }
            }
            carlog_ok!("Deleting", format!("🗑 {:?}", &target_path));
            match remove_dir_all(target_path.clone()) {
                Ok(_) => {
                    carlog_ok!("Deleted", format!("🗑 {:?}", &target_path));
                }
                Err(e) => {
                    carlog_error!(&format!(
                        "Failed to delete target path {:?}. Error: {}",
                        &target_path, e
                    ));
                    remove_dir_all(&shm_path)?;
                    exit(1);
                }
            }
        }
    }
    Ok((shm_path, target_path, shm_path_id, false))
}

pub fn mount(config: MountConfig) -> Result<()> {
    carlog_info!("Mounting", format!("Trying to mount {:?}", &config.target));
    let target = normalize_path(config.target);
    let (shm, target, _, linked) = prepare_tmpfs_path(target, config.copy_to)?;
    if !linked {
        match symlink(&shm, &target) {
            Ok(_) => {
                carlog_ok!("Linked", format!("⛓ {:?} -> {:?}", shm, target));
            }
            Err(e) => {
                carlog_error!(&format!(
                    "Failed to create symlink: {:?} -> {:?}. Error: {}",
                    &shm, &target, e
                ));
                remove_dir_all(&shm)?;
                exit(1);
            }
        }
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

            if config.copy_back {
                carlog_info!("Copying", "⏳ Start copying back. This may take a while...");

                // Here we copy back the data from the ramdisk to the original target path
                // We use the cp command because it preserves the file timestamps
                // This is important because cargo uses the file timestamps to determine if a file has changed
                // std::fs::copy does not preserve timestamps, so we use the cp command instead
                match std::process::Command::new("cp")
                    .arg("-r")
                    .arg("--preserve=mode,ownership,timestamps")
                    .arg(&link)
                    .arg(&target)
                    .output()
                {
                    Ok(_) => {
                        carlog_ok!("Copied back", format!("📁 {:?} -> {:?}", &link, &target));
                    }
                    Err(e) => {
                        carlog_error!(&format!(
                            "Failed to copy back data from ramdisk. Error: {}",
                            e
                        ));
                        exit(1);
                    }
                };
            }

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
        path
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
            copy_to: false,
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
            copy_back: false,
        })
        .expect("Failed to unmount test tmpfs");
        assert!(!target.exists());
    }

    #[test]
    fn test_copy_back() {
        let target = temp_dir().join("target");
        if target.exists() {
            remove_dir_all(target.clone()).expect("Failed to delete previous test target dir...");
        }

        // Create a test file
        std::fs::create_dir(target.clone()).expect("Failed to create test dir");
        std::fs::write(target.join("test.txt"), "test").expect("Failed to create test file");

        // Get the original timestamp
        let original_timestamp = std::fs::metadata(target.join("test.txt"))
            .unwrap()
            .modified()
            .unwrap();

        // Mount with copy_to
        let mntcfg = MountConfig {
            target: target.clone(),
            copy_to: true,
        };

        mount(mntcfg).expect("Failed to mount test tmpfs...");
        assert!(target.exists());
        let link = target.read_link();
        assert!(link.is_ok());
        let link = link.unwrap();
        assert!(link.starts_with(BASE_RAMDISK_FOLDER));

        // Check that the timestamp is the same
        let copied_timestamp = std::fs::metadata(link.join("test.txt"))
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(original_timestamp, copied_timestamp);

        // Modify the test file
        std::thread::sleep(std::time::Duration::from_millis(10)); // Sleep to make sure the timestamp changes
        std::fs::write(link.join("test.txt"), "test2").expect("Failed to write to test file");

        let modified_timestamp = std::fs::metadata(link.join("test.txt"))
            .unwrap()
            .modified()
            .unwrap();
        assert_ne!(original_timestamp, modified_timestamp);

        // Unmount with copy_back
        unmount(UnmountConfig {
            target: target.clone(),
            copy_back: true,
        })
        .expect("Failed to unmount test tmpfs");

        assert!(!link.exists());
        assert!(target.exists());

        let copied_back_timestamp = std::fs::metadata(target.join("test.txt"))
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(modified_timestamp, copied_back_timestamp);

        // Cleanup
        remove_dir_all(target).expect("Failed to delete previous test target dir...");
    }
}
