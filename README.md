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

Not usable for now **(work in progress)**.

## Requirements

### Cargo

We use _rustup/cargo_ to develop proot-rs, which is a common approach in Rust development. You can install them as shown [here](https://www.rust-lang.org/tools/install).

### cargo-make

We also use [`cargo-make`](https://github.com/sagiegurari/cargo-make) as build tool, which can help you launch complex compilation steps. It works a bit like `make`, and you can install it like this:

```shell
cargo install --force cargo-make
```

## Build

The recommended way is to use `cargo-make`:

```shell
cargo make build
```
The command basically consists of the following steps:
- Run `cargo build` on `loader-shim` package to compile the loader executable.
- Copy the loader executable `loader-shim` to `proot-rs/src/kernel/execve/loader-shim`
- Run `cargo build` on `proot-rs` package to  build the final executable.

If the compilation is successful, it should also print out the path to the `proot-rs` executable file.

### Build Release Version

To generate the release binary (it takes longer, but the binary generated is quicker), you can specify `--profile=production`:

```shell
cargo make build --profile=production
```

> Note: This [`--profile` option](https://github.com/sagiegurari/cargo-make#usage-profiles) comes from `cargo-make`, which has a different meaning than [the `profile` in cargo](https://doc.rust-lang.org/cargo/reference/profiles.html). And it is processed by `cargo-make` and will not be passed to `cargo`. 

### Cross Compilation

Currently `proot-rs` supports multiple platforms. You can change the compilation target by setting the environment variable `CARGO_BUILD_TARGET`.

For example, compiling to the `arm-linux-androideabi` target:
```shell
CARGO_BUILD_TARGET=arm-linux-androideabi cargo make build
```

<!-- TODO: Try to compile and test multiple targets in CI, and crate a table here. -->

## Run

Build and run `proot-rs`:

```shell
cargo make run -- "<args-of-proot-rs>"
```

Build and run release version of `proot-rs`:

```shell
cargo make run --profile=production -- "<args-of-proot-rs>"
```

## Tests

### Setup new rootfs for testing

Typically, we need to specify a new rootfs path for testing proot-rs.

This script provided below can be used to create one:

```shell
# This will create a busybox-based temporary rootfs at ./rootfs/
bash scripts/mkrootfs.sh
```
### Unit testing

Start running unit tests:

```shell
cargo make unit-test
```
> Note: Add the option `--profile=production` if you want to test a release build of proot-rs

By default, By default, `./rootfs/` will be used as the root filesystem for testing purposes. But you can set the environment variable `PROOT_TEST_ROOTFS` to change this behavior.

```shell
export PROOT_TEST_ROOTFS="<absolute-path-to-a-rootfs>"
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

