
## efi related linux applications.

### uefi-boot-menu

Present UEFI boot menu, set BootNext, optionally reboot.

### mini-bootcfg

Manage UEFI boot configuration for UKI boot.

Implements a subset of the `kernel-bootcfg` tool in rust (see
https://gitlab.com/kraxel/virt-firmware).  Plan is to allow dropping
the python dependency on uki cloud images.

### list-sb-vars

print secure boot certificate databases.

### generate-boot-csv

generate BOOT.CSV file for direct UKI boot.
