
## efi applications, mostly for testing

Can be started via `qemu-system-x86_64 -kernel ${app}.efi`.

### hello.efi

Print hello world.

### netboot.efi

Application to boot HTTP images (typically UKI or ISO).  Accepts an
URL on the command line, goes update boot configuration to load this
on next boot, resets the machine.

### fwcfg.efi

Print some qemu fwcfg info.

### pciscan.efi

Scan and list pci devices.
