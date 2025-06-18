#!/bin/bash

# args
mode="$1"

# config
qemu="$HOME/projects/qemu/build/default/qemu-system-x86_64"
wrap="$HOME/projects/virt-firmware-rs/target/debug/igvm-wrap"
show="$HOME/projects/virt-firmware-rs/target/debug/igvm-inspect"

case "$mode" in
    bios | seabios)
        firmware="/usr/share/seabios/bios-256k.bin"
        ccobject="nocc"
        wrapopts=""
        ;;
    efi | uefi | ovmf)
        firmware="/usr/share/edk2/ovmf/OVMF_CODE.cc.fd"
        ccobject="nocc"
        wrapopts=""
        ;;
    *)
        echo "unknown mode: $mode"
        exit 1
esac

work="$(mktemp -d /tmp/XXXXXXX)"
trap "rm -rf $work" EXIT
igvm="$work/firmware.igvm"
(
    set -ex
    $wrap -i "$firmware" -o "$igvm" $wrapopts
    $show "$igvm"
    $qemu -M q35 -m 1g -nographic \
          -object "$ccobject",id=ccguest \
          -object igvm-cfg,id=igvm,file="$igvm" \
          -machine igvm-cfg=igvm,confidential-guest-support=ccguest
)
