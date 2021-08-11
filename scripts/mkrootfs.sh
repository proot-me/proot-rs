#!/bin/sh

# This script is used to create rootfs directory, and is currently 
# only used for proot-rs testing. For now we only allow the creation 
# of busybox rootfs, more types of images will be introduced later.
# 
# The first parameter specifies the path to the rootfs directory, 
# if the target directory is not empty then we will skip generating.

PROOT_TEST_ROOTFS="./rootfs"

rootfs_type="busybox"

while getopts "d:t:" opt; do
  case $opt in
    d)
      PROOT_TEST_ROOTFS="${OPTARG}"
      ;;
    t)
      rootfs_type="${OPTARG}"
      ;;
    *)
      echo "Invalid option"
      exit 1
  esac
done

echo "Preparing rootfs...   type: ${rootfs_type}  path: ${PROOT_TEST_ROOTFS}"

rootfs_url=""
case ${rootfs_type} in
  busybox)
    case $(uname -m) in
      i386|i686)  rootfs_url="https://github.com/docker-library/busybox/raw/dist-i386/stable/glibc/busybox.tar.xz" ;;
      x86_64)  rootfs_url="https://github.com/docker-library/busybox/raw/dist-amd64/stable/glibc/busybox.tar.xz" ;;
      armv7l)  rootfs_url="https://github.com/docker-library/busybox/raw/dist-arm32v7/stable/glibc/busybox.tar.xz" ;;
      aarch64)  rootfs_url="https://github.com/docker-library/busybox/raw/dist-arm64v8/stable/glibc/busybox.tar.xz" ;;
      *)  echo "Unsupported architecture $(uname -m)"; exit 1 ;;
    esac
    ;;
  alpine)
    case $(uname -m) in
      i386|i686)  rootfs_url="https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86/alpine-minirootfs-3.14.0-x86.tar.gz" ;;
      x86_64)  rootfs_url="https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.0-x86_64.tar.gz" ;;
      armv7l)  rootfs_url="https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/armv7/alpine-minirootfs-3.14.0-armv7.tar.gz" ;;
      aarch64)  rootfs_url="https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/aarch64/alpine-minirootfs-3.14.0-aarch64.tar.gz" ;;
      *)  echo "Unsupported architecture $(uname -m)"; exit 1 ;;
    esac
    ;;
  *)      echo "Unknown rootfs type ${rootfs_type}"; exit 1 ;;
esac

if [ -n "$(ls -A "${PROOT_TEST_ROOTFS}" 2>/dev/null)" ]; then
  echo "The rootfs path ${PROOT_TEST_ROOTFS} exist but not empty. Skip creating rootfs..."
  exit 0
fi

echo "Creating ${rootfs_type} rootfs for $(uname -m) architecture in ${PROOT_TEST_ROOTFS}"

mkdir -p "${PROOT_TEST_ROOTFS}"

trap 'rm -f "${rootfs_archive}"' EXIT

rootfs_archive="$(mktemp)" || { echo "Failed to create temp file"; exit 1; }

wget -O "${rootfs_archive}" "${rootfs_url}" || { echo "Failed to download ${rootfs_type} archive"; exit 1; }

tar -C "${PROOT_TEST_ROOTFS}" -xf "${rootfs_archive}" || { echo "Failed to unpack ${rootfs_type} tarball. Maybe the file is broken"; exit 1; }

echo "The rootfs was created at ${PROOT_TEST_ROOTFS}"

