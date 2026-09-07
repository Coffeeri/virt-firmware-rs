#!/bin/sh

# https://github.com/microsoft/secureboot_objects.git
sbrepo="$HOME/projects/secureboot"

srcdir="${sbrepo}/PostSignedObjects/SignedByKEK2023"
dstdir="libefi/src/sb/dbx"

test -d "$srcdir" || exit 1

(set -x; cd "$sbrepo"; git pull)
mkdir -p "${dstdir}"
cp -v "${srcdir}/dbx_x64.efiauth2"     "${dstdir}/2023.x86_64.auth"
cp -v "${srcdir}/dbx_aarch64.efiauth2" "${dstdir}/2023.aarch64.auth"
