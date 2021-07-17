# Integration Testing

This document describes the integration testing part of proot-rs. It is intended for developers to understand the current status of integration testing in proot-rs.


## Why and how to

proot-rs currently uses unit testing and integration testing to find potential software faults.

For unit testing, we use `cargo test` to ensure the correctness of the functions, in a similar way to the normal rust program. However, we still need a way to be able to test the whole program, and that is the purpose of integration testing.


We use [Bats](https://github.com/bats-core/bats-core) to run integration tests. Bats is a Bash-based testing framework that allows developers to write simple Bash scripts to test their command-line programs.

We also considered ShellSpec and shUnit2, but they seem to be more suitable for testing shell scripts rather than command line programs.


## Run tests

### Run manually

To launch integration testing, you need to [install bats](https://github.com/bats-core/bats-core/blob/master/docs/source/installation.rst) first, and make sure `bats` is in your `PATH`.

> Note: Before running integration tests, you need to set `PROOT_TEST_ROOTFS` to the path of new rootfs, or the `rootfs` directory in the root of the project will be used as the new rootfs.


Before you start, make sure you are in the root directory of this project.

1. Compile proot-rs

    ```shell
    cargo build
    ```
    This will generate a executable file at `target/debug/proot-rs`, which will be used in during integration tests.

2. Run all test scripts under the `tests/` directory
    ```shell
    bats -r tests
    ```

### Run in Docker

TODO

### Run in CI

Integration tests are now added to github [workflow](.github/workflows/tests.yml)


## Write tests

All integration test scripts are placed in the `tests/` directory. They all use `.bats` as suffix and are named as the category of the tests. A script file may contains more than one test case.

`helper.bash` is a Bash script which is included by all test scripts, which provided some helper functions and global variables.

We usually use proot-rs to run a shell script to test it's correctness, i.e. (`proot-rs -- /bin/sh -x -e -c " #some tests"`).  But for some tests that cannot be written in shell script, we can also write them in C.


For more information, please read [Bats: Writing tests](https://bats-core.readthedocs.io/en/stable/writing-tests.html)




