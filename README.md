# proot-rs

[![Tests](https://github.com/proot-me/proot-rs/actions/workflows/tests.yml/badge.svg)](https://github.com/proot-me/proot-rs/actions/workflows/tests.yml)

_Rust implementation of PRoot, a ptrace-based sandbox._

**(Work in progress)**

`proot-rs` works by intercepting all Linux system calls that use paths (`execve`, `mkdir`, `ls`, ...)
and translating these with the specified path bindings, in order to simulate `chroot`,
and all this without requiring admin rights (`ptrace` do not require any special rights).

So for instance, this command:

```
proot-rs -R /home/user/ mkdir /myfolder
```

(`-R` defines a new root and adds usual bindings like `/bin`)

will be equivalent to:

```
mkdir /home/user/myfolder/
```

Hence, you can apply `proot-rs` to a whole program in order to sandbox it.
More concretely, you can for instance download a docker image, extract it,
and run it, without needing docker:

```
proot-rs -R ./my-docker-image /bin/sh
```

## Usage

```
proot-rs --help
proot-rs 

USAGE:
    proot-rs [OPTIONS] [--] [command]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bind <bind>...     Make the content of *host_path* accessible in the guest rootfs. Format:
                             host_path:guest_path
    -w, --cwd <cwd>          Set the initial working directory to *path*. [default: /]
    -r, --rootfs <rootfs>    Use *path* as the new guest root file-system. [default: /]

ARGS:
    <command>...    
```

## Requirements

Use the nightly Rust channel for rustc:

```
rustup default nightly
```

Some dependencies (like `syscall`) depend on features (`asm` in this case) that are not 
on the stable channel yet.

## Build

The recommended way is to use _rustup/cargo_:

```text
cargo build
```

It will install all the dependencies and compile it (in debug mode).

To generate the release binary (it takes longer, but the binary generated is quicker):

```text
cargo build --release
```

## Tests

### Setup new rootfs for testing

Typically, we need to specify a new rootfs path for testing proot-rs.

This script provided below can be used to create one:

```sh
# This will create a busybox-based temporary rootfs at ./rootfs/
bash scripts/mkrootfs.sh
```

Then set an environment variable `PROOT_TEST_ROOTFS` so the test program can find it:

```sh
export PROOT_TEST_ROOTFS=./rootfs/
```

If you want to use the same rootfs as the host, just set it to `/`:

```sh
export PROOT_TEST_ROOTFS=/
```

### Unit testing

> Note: When running unit tests, it is required that `PROOT_TEST_ROOTFS` must be set

Start running tests:

```shell
cargo test
```

### Integration testing

For the section on running integration tests, please read the [Integration Testing documentation](./tests/README.md)

## Contributing

We use git hooks to check files staged for commit to ensure the consistency of Rust code style.

Before you start, please run the following command to setup git hooks:

```shell
git config core.hooksPath .githooks
```

To format code manually:

```shell
cargo fmt
```

