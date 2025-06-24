#!/bin/bash

arch=$(uname -m)

case "$arch" in
    aarch64)
        efiarch="aa64"
        ;;
    x86_64)
        efiarch="x64"
        efishell="/usr/share/edk2/ovmf/Shell.efi"
        ;;
    *)
        echo "unknown arch: $arch"
        exit 1
        ;;
esac

if test ! -d "/sys/firmware/efi"; then
    echo "no UEFI system"
    exit 1
fi

if test ! -f "$efishell"; then
    echo "no EFI shell found"
    exit 1
fi

ESP="$(bootctl -p)"
if test ! -d "${ESP}/EFI"; then
    echo "no ESP found"
    exit 1
fi

echo "# install shell.efi"
cp -v "$efishell" "${ESP}/shell${efiarch}.efi"	|| exit 1

echo "# install efi utils"
mkdir -p ${ESP}/utils				|| exit 1
for util in /usr/share/virt-firmware-rs/$efiarch/*.efi; do
    cp -v "$util" "${ESP}/utils/${util##*/}"	|| exit 1
done

if test -x "$(which mini-bootcfg 2>/dev/null)"; then
    echo "# add shell to uefi boot menu"
    mini-bootcfg \
        --loglevel info \
        --add-uki "${ESP}/shell${efiarch}.efi" \
        --title "EFI Shell @ ESP" \
        --boot-order last
fi
