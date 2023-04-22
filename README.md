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
#### Copying data back to disk
In order to copy data back to disk you can use the flag `-c` or `--copy-to` in the `mount` and `unmount` subcommands.

#### Sub-commands usage
Cargo ramdisk has three main subcommands for its operation each one with its options and flags.

##### mount
```
Mount a ramdisk, same as not specifying a subcommand

USAGE:
    cargo ramdisk mount [FLAGS] [OPTIONS]

FLAGS:
    -c, --copy-to    Copy the contents of the target folder to the ramdisk
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --target <target>    The path to the target folder where compilation output is written [default: ./target]
```

##### remount
```
Remount an existing ramdisk

USAGE:
    cargo ramdisk remount [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --target <target>    The path to the target folder where compilation output is written [default: target]
```

##### unmount
```
Unmount an existing ramdisk

USAGE:
    cargo ramdisk unmount [FLAGS] [OPTIONS]

FLAGS:
    -c, --copy-back    Copy back the contents of the ramdisk to the target folder
    -h, --help         Prints help information
    -V, --version      Prints version information

OPTIONS:
    -t, --target <target>    The path to the target folder where compilation output is written [default: target]
```
