
## igvm related linux applications.

### igvm-inspect

Inspect igvm files, print informations to the console.

### igvm-wrap

Wrap traditional firmware images (flash/rom) into an igvm.

`igvm-wrap --tdx` wraps TDVF using standard IGVM directives and embedded
TDVF metadata for a TDVF-aware loader. See [the TDVF boot contract](../docs/tdvf-boot.md)
for loader requirements and limitations.

### igvm-update

Update OVMF igvm images (created by igvm-wrap), add efi binary
(typically an UKI) and optionally a shim binary for direct kernel
boot.  Requires edk2-stable202511 or newer.

### igvm-elfis

Convert elf binaries (as created by mefisto crate) into igvm files.
