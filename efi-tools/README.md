
## efi related linux applications.

### uefi-boot-menu

Present UEFI boot menu, set BootNext, optionally reboot.

### mini-bootcfg

Manage UEFI boot configuration for UKI boot.

Implements a subset of the `kernel-bootcfg` tool in rust (see
https://gitlab.com/kraxel/virt-firmware).  Plan is to allow dropping
the python dependency on uki cloud images.

### list-sb-vars

Print secure boot certificate databases.

### enroll-sb-vars

Enroll secure boot certificate databases and enable secure boot.
Requires recent (ekd2-stable202605 or newer) OVMF build.  See manpage
or `--help` for details.

### check-sb-signature

Verify secure boot signatures.  Supports both signed variable updates
(for example the dbx updates microsoft ships) and authenticode signed
binaries.

### generate-boot-csv

Generate BOOT.CSV file for direct UKI boot.
