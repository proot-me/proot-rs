# proot-rs [![](https://github.com/proot-me/proot-rs/workflows/Rust/badge.svg)](https://github.com/proot-me/proot-rs/actions)


Rust implementation of PRoot, a ptrace-based sandbox.
_(Work in progress)_

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

Hence, you can apply `proot-rs` to a whole program in order sandbox it.
More concretely, you can for instance download a docker image, extract it,
and run it, without needing docker:
```
proot-rs -R ./my-docker-image /bin/sh
```

## Usage
Not usable for now _(work in progress)_.

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
Simply run:
```
cargo test
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
