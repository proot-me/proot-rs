#!/bin/sh

# This script is used to create rootfs directory, and is currently 
# only used for proot-rs testing. For now we only allow the creation 
# of busybox rootfs, more types of images will be introduced later.
# 
# The first parameter specifies the path to the rootfs directory, 
# if the target directory is not empty then we will skip generating.
 
rootfs="$1"

arch=""
case $(uname -m) in
    i386)   arch="i386" ;;
    i686)   arch="i386" ;;
    x86_64) arch="amd64" ;;
    *)      echo "Unsupported architecture $(uname -m)"; exist 1 ;;
esac

if [ ! -z "$(ls -A ${rootfs} 2>/dev/null)" ]; then
    echo "The rootfs path ${rootfs} exist but not empty. Skip creating rootfs..."
    exit 0
fi

echo "Creating busybox rootfs for ${arch} architecture in ${rootfs}"

mkdir -p $rootfs

trap 'rm -f "$rootfs_tarball"' EXIT
rootfs_tarball=$(mktemp) || { echo "Failed to create temp file"; exit 1; }
curl -o $rootfs_tarball -L -C - "https://github.com/docker-library/busybox/raw/dist-${arch}/stable/glibc/busybox.tar.xz" || { echo "Failed to download busybox tarball"; exit 1; }
tar -C $rootfs -xf $rootfs_tarball || { echo "Failed to unpack busybox tarball. Maybe the file is broken"; exit 1; }

echo "The rootfs was created at ${rootfs}"