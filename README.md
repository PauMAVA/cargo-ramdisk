# cargo-ramdisk
<b>This crate is only supported for unix like systems!</b>
`cargo-ramdisk` creates a ramdisk at the target folder of your project for ridiculously faster compilation times.

This is achieved without root permisions by linking your target folder to a temporary folder in `/dev/shm` in your unix like OS. This location is `rw` for all users and is mounted in virtual memory as a `tmpfs`.

### Install
```
cargo install cargo-ramdisk
```

### Usage
```
USAGE:
    cargo ramdisk [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --target <target>    The path to the target folder where compilation output is written [default: target/]

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    mount      Mount a ramdisk, same as not specifying a subcommand
    remount    Remount an existing ramdisk
    unmount    Unmount an existing ramdisk
```
