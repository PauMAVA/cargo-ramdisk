extern crate structopt;

mod config;

use config::{CargoRamdiskConfig, MountConfig, RemountConfig, Subcommands, UnmountConfig};
use structopt::StructOpt;

fn main() {
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

fn mount(_config: MountConfig) {}

fn remount(_config: RemountConfig) {}

fn unmount(_config: UnmountConfig) {}
