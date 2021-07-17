#!/usr/bin/env bats

load helper


@test "test terminate tracees after proot-rs exits" {
    runp proot-rs --rootfs "$ROOTFS" -- /bin/sh -c '/bin/kill -11 $PPID; /bin/echo "The tracee is still alive, which is not allowed";'
    [[ "$output" != *"still alive"* ]]
}
