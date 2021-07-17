#!/usr/bin/env bats

load ../helper


@test "test execute dynamically linked binary" {
    # compile test case
    compile_c_dynamic "$ROOTFS/bin/test_execve_dynamic" "$BATS_TEST_DIRNAME/test_execve.c"
    runp proot-rs --rootfs "$ROOTFS" --cwd / -- /bin/test_execve_dynamic
    # remember to delete the binary file
    rm "$ROOTFS/bin/test_execve_dynamic"
    [ "$status" -eq 0 ]
}


@test "test execute statically linked binary" {
    # compile test case
    compile_c_static "$ROOTFS/bin/test_execve_static" "$BATS_TEST_DIRNAME/test_execve.c"
    runp proot-rs --rootfs "$ROOTFS" --cwd / -- /bin/test_execve_static
    # remember to delete the binary file
    rm "$ROOTFS/bin/test_execve_static"
    [ "$status" -eq 0 ]
}