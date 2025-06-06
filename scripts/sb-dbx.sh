#!/bin/sh

# https://github.com/microsoft/secureboot_objects.git
sbrepo="$HOME/projects/secureboot"

srcdir="${sbrepo}/PostSignedObjects/DBX"
dstdir="libefi/src/sb/dbx"

test -d "$srcdir" || exit 1

(set -x; cd "$sbrepo"; git pull)
mkdir -p "${dstdir}"
cp -v "${srcdir}/amd64/DBXUpdate.bin" "${dstdir}/x86_64.auth"
cp -v "${srcdir}/arm64/DBXUpdate.bin" "${dstdir}/aarch64.auth"
