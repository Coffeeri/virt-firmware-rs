
## library to read + write efi data structures

Implemented mostly what is needed to manage efi boot configuration,
i.e. BootNNNN. BootOrder, etc variables.

Also some support for secure boot variables.  Parse authenticated
variable updates.  Parse efi signature databases.

Support to read/write efi variables via efivarfs
(`/sys/firmware/efi/efivars`).

## Features

* `std`: Enable support for the standard library.

* `udev`: Enable support for udev.  Needed to map linux file names to
  efi device paths.

* `sbdata`: Include secure boot data: certificates for `KEK`
  and `db`, revocation databases for `dbx`.

* `sbtest`: Include `Red Hat Test Certifying CA` certificate.
