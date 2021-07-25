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


@test "test nested calls between fork() / vfork() / clone()" {
    # compile test case
    compile_c_dynamic "$ROOTFS/bin/nested_fork_vfork_clone" "$BATS_TEST_DIRNAME/nested_fork_vfork_clone.c"
    runp proot-rs --rootfs "$ROOTFS" --cwd / -- /bin/nested_fork_vfork_clone
    # remember to delete the binary file
    rm "$ROOTFS/bin/nested_fork_vfork_clone"
    [ "$status" -eq 0 ]
    [ "${lines[0]}" = "111 211 311 121 221 321 131 231 331 112 212 312 122 222 322 132 232 332 113 213 313 123 223 323 133 233 333 " ]
    [ "${#lines[@]}" -eq 1 ]
}

