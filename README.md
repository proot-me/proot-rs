# proot-rs

**Please take the PRoot Usage Survey for 2023!** [![Survey](https://img.shields.io/badge/survey-2023-green?style=flat-square)](https://www.surveymonkey.com/r/7GVXS7W)

--

[![Tests](https://img.shields.io/github/actions/workflow/status/proot-me/proot-rs/tests.yml?style=flat-square)](https://github.com/proot-me/proot-rs/actions/workflows/tests.yml)
[![Releases](https://img.shields.io/github/v/release/proot-me/proot-rs?sort=semver&style=flat-square)](https://github.com/proot-me/proot-rs/releases)


_Rust implementation of PRoot, a ptrace-based sandbox._

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
proot-rs 0.1.0
chroot, mount --bind, and binfmt_misc without privilege/setup.

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

### Cargo

We use _rustup/cargo_ to develop proot-rs, which is a common approach in Rust development. You can install them as shown [here](https://www.rust-lang.org/tools/install).

### cargo-make

We also use [`cargo-make`](https://github.com/sagiegurari/cargo-make) as build tool, which can help you launch complex compilation steps. It works a bit like `make`, and you can install it like this:

> Note: We recommend using the stable toolchain to install `cargo-make` in order to avoid installation failures

```shell
# Install stable rust toolchain
rustup toolchain install stable
# Install cargo-make
cargo +stable install --force cargo-make
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

#### With [`cross`](https://github.com/rust-embedded/cross) (Recommended)

The `cross` is a “zero setup” cross compilation and “cross testing” tool, which uses docker to provide an out-of-the-box cross-compilation environment which contains a ready-to-use cross-compilation toolchain. So we don't need to prepare it ourselves.

> Note that `cross` depends on docker, so you need to install docker and start it.

- To use cross, you may need to install it first:

    ```shell
    cargo install cross
    ```

- Run with `USE_CROSS=true`

  Our `Makefile.toml` script contains the integration with `cross`.

  For example, to compile to the `arm-linux-androideabi` target, you can simply run:
  ```shell
  USE_CROSS=true CARGO_BUILD_TARGET=arm-linux-androideabi cargo make build
  ```
  > The `USE_CROSS=true` will indicate the build script to use the `cross` tool to compile.

#### With `cargo` (Native Approach)

You can also use the rust native approach to cross-compile proot-rs.

For example, to compile to the `arm-linux-androideabi` target
- Install this target first:
  ```shell
  rustup target add arm-linux-androideabi
  ```
- Cross compile with `cargo`
  ```shell
  CARGO_BUILD_TARGET=arm-linux-androideabi cargo make build
  ```
  > Note: This command may fail for compiling to some targets because the linker reports some error. In this case, You may need to install an additional gcc/clang toolchain on your computer, and specify the appropriate linker path in the `.cargo/config.toml` file

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
export PROOT_TEST_ROOTFS="`realpath ./rootfs/`"
cargo test --package=proot-rs -- --test-threads=1
```

> Note:
> - Since our testing will spawn multiple processes, we need `--test-threads=1` to avoid deadlock caused by `fork()`. The option `--nocapture` may also be needed to show the original panic reason printed out by the child process.
> - Add the option `--profile=production` if you want to test a release build of proot-rs

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

