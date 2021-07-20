#!/usr/bin/env bats

load helper


@test "test bind dir to dir" {
    # bind /etc to /home
    proot-rs --bind "/etc:/home" -- /bin/sh -c "/bin/diff /etc /home"
}


@test "test bind file to file" {
    # bind /etc/group to /etc/passwd
    proot-rs --bind "/etc/group:/etc/passwd" -- /bin/sh -c "/bin/diff /etc/group /etc/passwd"
}


@test "test bind dir to file" {
    # bind /home to /etc/passwd
    # this may seem odd, but it is allowed
    proot-rs --bind "/home:/etc/passwd" -- /bin/sh -c "/bin/diff /home /etc/passwd"
}


@test "test bind file to dir" {
    # bind /etc/passwd to /home
    # this may seem odd, but it is allowed
    proot-rs --bind "/etc/passwd:/home" -- /bin/sh -c "/bin/diff /etc/passwd /home"
}


# Will be removed after the implementation of bind glue
@test "test bind target must exist" {
    runp proot-rs --bind "/etc/passwd:/etc/non_exist_path" -- /bin/sh -c "/bin/true"
    [ $status -ne 0 ]
}


@test "test --bind with getdents64() results" {
    mkdir "$ROOTFS/tmp/test_bind_with_getdents64"
    echo "first" > "$ROOTFS/tmp/test_bind_with_getdents64/file1"
    echo "second" > "$ROOTFS/tmp/test_bind_with_getdents64/file2"
    chmod 644 "$ROOTFS/tmp/test_bind_with_getdents64/file1"
    chmod 777 "$ROOTFS/tmp/test_bind_with_getdents64/file2"
    # bind "$ROOTFS/tmp/test_bind_with_getdents64/file1" to "/tmp/test_bind_with_getdents64/file2"
    runp proot-rs --rootfs "$ROOTFS" --bind "$ROOTFS/tmp/test_bind_with_getdents64/file1:/tmp/test_bind_with_getdents64/file2" -- /bin/sh -e -c ' \
        PATH=/bin
        # Get the output of ls -l and filter out the lines related to file1 and file2
        output=$(ls -l /tmp/test_bind_with_getdents64 | grep "file" | sed  "s/file.*//g") \
        # The $output should contain two lines
        [ "$(echo $output | wc -l)" -eq 2 ]
        # And their attributes should be the same.
        [ "$(echo $output | sort | uniq | wc -l)" -eq 1 ]
    '
    rm -rf "$ROOTFS/tmp/test_bind_with_getdents64"
    [ "$status" -eq 0 ]
}