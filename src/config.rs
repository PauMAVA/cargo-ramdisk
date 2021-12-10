use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cargo-ramdisk",
    about = "Create target folder as a ramdisk for faster Rust compilation."
)]
pub struct CargoRamdiskConfig {
    /// The path to the target folder where compilation output is written
    #[structopt(default_value = "target/", short, long)]
    pub target: PathBuf,

    #[structopt(subcommand)]
    pub subcommand: Option<Subcommands>,
}

#[derive(Debug, StructOpt)]
pub struct MountConfig {
    /// The path to the target folder where compilation output is written
    #[structopt(default_value = "./target", short, long)]
    pub target: PathBuf,
}

impl From<CargoRamdiskConfig> for MountConfig {
    fn from(conf: CargoRamdiskConfig) -> Self {
        Self {
            target: conf.target,
        }
    }
}

impl From<&RemountConfig> for MountConfig {
    fn from(config: &RemountConfig) -> Self {
        Self {
            target: config.target.clone(),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct RemountConfig {
    /// The path to the target folder where compilation output is written
    #[structopt(default_value = "target", short, long)]
    pub target: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct UnmountConfig {
    /// The path to the target folder where compilation output is written
    #[structopt(default_value = "target", short, long)]
    pub target: PathBuf,
}

impl From<&RemountConfig> for UnmountConfig {
    fn from(config: &RemountConfig) -> Self {
        Self {
            target: config.target.clone(),
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum Subcommands {
    /// Mount a ramdisk, same as not specifying a subcommand
    Mount(MountConfig),

    /// Remount an existing ramdisk
    #[structopt(name = "remount")]
    Remount(RemountConfig),

    /// Unmount an existing ramdisk
    #[structopt(name = "unmount")]
    Unmount(UnmountConfig),
}
