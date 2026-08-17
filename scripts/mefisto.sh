#!/bin/bash

# input
version="$(uname -r)"
kernel="/boot/vmlinuz-${version}"
initrd="/boot/initramfs-${version}.img"
append="$(cat /etc/kernel/cmdline)"

# output
uki="$(mktemp ${TMPDIR-/var/tmp}/${version}-XXXXXXXX.uki.efi)"
wrap="$(mktemp ${TMPDIR-/var/tmp}/${version}-XXXXXXXX.wrap.igvm)"
igvm="$(mktemp ${TMPDIR-/var/tmp}/${version}-XXXXXXXX.igvm)"
trap "echo '# cleanup'; rm -f $uki $wrap $igvm" EXIT

# generate uki
echo "# generate uki"
(set -x;
 ukify build \
       --linux "$kernel" \
       --initrd "$initrd" \
       --cmdline "$append" \
       --uname "$version" \
       --output "$uki")		|| exit 1

# generate igvm
case "$0" in
    *mefisto*)
        echo "# generate igvm, using mefisto"
        tag="mefisto"
        (set -x;
         target/debug/igvm-elfis \
             --input target/x86_64-unknown-none/debug/mefisto \
             --kernel "$uki" \
             --output "$igvm")	|| exit 1
        ;;
    *stateless*)
        echo "# generate igvm, using stateless ovmf"
        tag="stateless"
        (set -x;
         target/debug/igvm-wrap \
             --input /usr/share/edk2/ovmf/OVMF.stateless.fd \
             --output "$wrap" && \
         target/debug/igvm-update \
             --input "$wrap" \
             --kernel "$uki" \
             --add-hash \
             --output "$igvm")	|| exit 1
        ;;
esac
echo "# inspect igvm"
target/debug/igvm-inspect "$igvm"

# copy files to ESP
echo "# copy files to ESP"
destdir="$(bootctl --print-esp)/igvm"
outfile="${destdir}/${version}.${tag}.igvm"
mkdir -p "$destdir"
cp -v /usr/share/edk2/ovmf/Shell.efi "${destdir%/igvm}/shellx64.efi"
cp -v target/x86_64-unknown-uefi/debug/vmfwupdate.efi "$destdir"
cp -v "$igvm" "$outfile"

# configure grub
echo "# configure grub"
cat <<EOF | tee /boot/grub2/custom.cfg
menuentry 'UEFI Shell' {
	search --file --set=esp /shellx64.efi
	chainloader (\$esp)/shellx64.efi
}
EOF
for igvm in $destdir/*.igvm; do
cat <<EOF | tee --append /boot/grub2/custom.cfg
menuentry '${igvm##*/}' {
	search --file --set=esp /igvm/vmfwupdate.efi
	chainloader (\$esp)/igvm/vmfwupdate.efi \\\\igvm\\\\${igvm##*/}
}
EOF
done
