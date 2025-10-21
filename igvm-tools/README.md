
## igvm related linux applications.

### igvm-inspect

Inspect igvm files, print informations to the console.

### igvm-wrap

Wrap traditional firmware images (flash/rom) into an igvm.

### igvm-update

Update OVMF igvm images (created by igvm-wrap), add efi binary
(typically an UKI) and optionally a shim binary for direct kernel
boot.  Requires edk2-stable202511 or newer.

### igvm-elfis

Convert elf binaries (as created by bios-64 crate) into igvm files.
