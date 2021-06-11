# proot-rs

[![](https://github.com/proot-me/proot-rs/workflows/Rust/badge.svg)](https://github.com/proot-me/proot-rs/actions)

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

Not usable for now **(work in progress)**.

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

Start running tests:

```shell
# Limit the number of threads running the test case to 1 to avoid deadlock problems caused by using fork in a multi-threaded environment
cargo test -- --test-threads=1
```

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

