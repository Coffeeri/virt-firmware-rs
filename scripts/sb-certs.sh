#!/bin/sh

srcdir="/usr/share/virt-sb-certs"
dstdir="libefi/src/sb/certs"

test -d "$srcdir" || exit 1

certs=$(find $srcdir -name '*.pem' | grep -E -e '(uefi|kek|windows)')
for cert in $certs; do
    name="${cert#$srcdir/}"
    sdir="${name%/*}"
    file="${name##*/}"
    base="${file%.pem}"
    echo "# $sdir $base"
    mkdir -p "${dstdir}/${sdir}"
    openssl x509 -in "$cert" -out "${dstdir}/${sdir}/${base}.der" -outform DER 
done
