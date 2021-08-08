# Integration Testing

This document describes the integration testing part of proot-rs. It is intended for developers to understand the current status of integration testing in proot-rs.

## Why and How To

proot-rs currently uses unit testing and integration testing to find potential software faults.

For unit testing, we use `cargo test` to ensure the correctness of the functions in a similar way to the normal rust program. However, we still need a way to be able to test the whole program, and that is the purpose of integration testing.

We use [Bats](https://github.com/bats-core/bats-core) to run integration tests. Bats is a Bash-based testing framework that allows developers to write simple Bash scripts to test their command-line programs.

We also considered ShellSpec and shUnit2, but they seem to be more suitable for testing shell scripts rather than command line programs.

## Run Tests

### Run Manually

To launch integration testing, you need to [install bats](https://github.com/bats-core/bats-core/blob/master/docs/source/installation.rst) first, and make sure that `bats` is in your `PATH`.

We use `cargo-make` to manage the build tasks, and you can launch integration test directly by:
```shell
cargo make integration-test
```
> Add the option `--profile=production` if you want to test a release build of proot-rs

This command above will first rebuild `proot-rs` and then run `bats -r tests` to start the tests.

#### Environment Variables

There are two optional environment variables related to testing:
- `PROOT_TEST_ROOTFS`: Absolute path of a guest rootfs for testing purposes.
- `PROOT_RS`: Absolute path of the proot-rs executable file to be tested.


### Run in Docker

We provide a [Dockerfile](./Dockerfile) for running integration tests in docker.

First go to the root of the project, and build docker image from Dockerfile

```shell
docker build -f tests/Dockerfile -t proot/proot-rs-test:latest .
```

Then, start a container to run the test:

```shell
docker run --rm -it proot/proot-rs-test:latest
```

### Run in CI

Integration tests are now added to the GitHub [workflow](.github/workflows/tests.yml)

## Write Tests

All integration test scripts are placed in the `tests/` directory. They all use `.bats` as suffix and are named as the category of the tests. A script file may contains more than one test case.

The file `helper.bash` is a Bash script which is included by all test scripts, and provides some helper functions and global variables.

We usually use `proot-rs` to run a shell script to test for correctness, i.e. (`proot-rs -- /bin/sh -x -e -c " #some tests"`). But for some tests that cannot be written in shell script, we can also write them in C.

For more information, please read [Bats: Writing tests](https://bats-core.readthedocs.io/en/stable/writing-tests.html)

