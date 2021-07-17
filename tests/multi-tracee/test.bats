#!/usr/bin/env bats

load ../helper


@test "test clone with CLONE_FS set" {
    # compile test case
    compile_c_dynamic "$ROOTFS/bin/clone_with_clone_fs" "$BATS_TEST_DIRNAME/clone_with_clone_fs.c"
    runp proot-rs --rootfs "$ROOTFS" --cwd / -- /bin/clone_with_clone_fs
    # remember to delete the binary file
    rm "$ROOTFS/bin/clone_with_clone_fs"
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "/" ]
    # both the cwd of the child process and the cwd of the parent process are changed to /etc
    [ "${lines[1]}" = "/etc" ]
    [ "${lines[2]}" = "/etc" ]
    [ "${#lines[@]}" -eq 3 ]
}

@test "test clone without CLONE_FS set" {
    # by default, child process spawned in shell does not contains `CLONE_FS`
    runp proot-rs --rootfs "$ROOTFS" --cwd / -- /bin/sh -c "/bin/pwd -P; \
        /bin/sh -c 'cd /etc; /bin/pwd -P;'; \
        /bin/pwd -P;"
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "/" ]
    [ "${lines[1]}" = "/etc" ]
    [ "${lines[2]}" = "/" ]
    [ "${#lines[@]}" -eq 3 ]
}
