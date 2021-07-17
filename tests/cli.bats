#!/usr/bin/env bats

load helper


@test "test proot-rs --version" {
    proot-rs --version
}

@test "test proot-rs with default options" {
    proot-rs /bin/true
}

@test "test proot-rs options --cwd" {
    proot-rs --cwd /bin -- ./true
    proot-rs -w /bin -- ./true
}

@test "test proot-rs options --rootfs" {
    proot-rs --rootfs "$ROOTFS" -- /bin/true
    proot-rs -r "$ROOTFS" -- /bin/true
}

@test "test proot-rs options --cwd and --rootfs" {
    proot-rs --rootfs "$ROOTFS" --cwd /bin -- ./true
}

@test "test proot-rs options --bind" {
    proot-rs --bind "/etc:/home" -- /bin/stat /home/passwd
    proot-rs -b "/etc:/home" -- /bin/stat /home/passwd
}

