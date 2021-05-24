#!/bin/sh

# This script is used to create rootfs directory, and is currently 
# only used for proot-rs testing. For now we only allow the creation 
# of busybox rootfs, more types of images will be introduced later.
# 
# The first parameter specifies the path to the rootfs directory, 
# if the target directory is not empty then we will skip generating.

PROOT_TEST_ROOTFS="./rootfs"

while getopts "d:" opt; do
  case $opt in
    d)
      PROOT_TEST_ROOTFS="${OPTARG}"
      ;;
    *)
      echo "Invalid option"
      exit 1
  esac
done

arch=""
case $(uname -m) in
    i386)   arch="i386" ;;
    i686)   arch="i386" ;;
    x86_64) arch="amd64" ;;
    *)      echo "Unsupported architecture $(uname -m)"; exist 1 ;;
esac

if [ -n "$(ls -A "${PROOT_TEST_ROOTFS}" 2>/dev/null)" ]; then
    echo "The rootfs path ${PROOT_TEST_ROOTFS} exist but not empty. Skip creating rootfs..."
    exit 0
fi

echo "Creating busybox rootfs for ${arch} architecture in ${PROOT_TEST_ROOTFS}"

mkdir -p "${PROOT_TEST_ROOTFS}"

trap 'rm -f "${rootfs_archive}"' EXIT

rootfs_archive="$(mktemp)" || { echo "Failed to create temp file"; exit 1; }

curl -o "${rootfs_archive}" -L -C - "https://github.com/docker-library/busybox/raw/dist-${arch}/stable/glibc/busybox.tar.xz" || { echo "Failed to download busybox archive"; exit 1; }

tar -C "${PROOT_TEST_ROOTFS}" -xf "${rootfs_archive}" || { echo "Failed to unpack busybox tarball. Maybe the file is broken"; exit 1; }

echo "The rootfs was created at ${PROOT_TEST_ROOTFS}"

