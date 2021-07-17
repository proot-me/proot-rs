#!/bin/bash

# The root directory where the integration test files are placed
TEST_ROOT=$(dirname "$(readlink -f "$BASH_SOURCE")")

# The root directory of this project
PROJECT_ROOT="$TEST_ROOT/../"

# Path to the proot-rs binary
PROOT_RS="$PROJECT_ROOT/target/debug/proot-rs"

# Set the default path to the new rootfs, which is created in advance by `scripts/mkrootfs.sh`
# Note that if `PROOT_TEST_ROOTFS` is set, then the value of `ROOTFS` will the same as it; otherwise, the default value of `ROOTFS` is `$PROJECT_ROOT/rootfs`
[[ -z "${PROOT_TEST_ROOTFS}" ]] && ROOTFS="$PROJECT_ROOT/rootfs" || ROOTFS="${PROOT_TEST_ROOTFS}"

# A wrapper for bats' built-in `run` command.
# This function will first execute the original `run` command, and then print the `$status` and `$output` to the `stderr`.
# One advantage over `run` is that the results of the command will be displayed when the test fails, making it easier for developers to debug.
function runp() {
    run "$@"
    echo "command: $@" >&2
    echo "status:  $status" >&2
    echo "output:  $output" >&2
}

# A wrapper function for proot-rs binary.
function proot-rs() {
    "$PROOT_RS" "$@"
}

# Compile a single C source code file ($2) to statically linked binary ($1)
function compile_c_static() {
    local target_path=$1
    local source_path=$2
    gcc -static -o "$target_path" "$source_path"
}

# Same as `compile_c_static()`, but the final binary is dynamically linked
function compile_c_dynamic() {
    local target_path=$1
    local source_path=$2
    gcc -o "$target_path" "$source_path"
}
