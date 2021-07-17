#!/usr/bin/env bats

load helper


@test "test get and set cwd" {
    run proot-rs --rootfs "$ROOTFS" --cwd /bin -- /bin/sh -c "pwd -P; cd /; pwd -P; cd /etc/; pwd -P"
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "/bin" ]
    [ "${lines[1]}" = "/" ]
    [ "${lines[2]}" = "/etc" ]
    [ "${#lines[@]}" -eq 3 ]
}
