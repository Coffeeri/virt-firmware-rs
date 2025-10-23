
## efi applications, mostly for testing

Can be started via `qemu-system-x86_64 -kernel ${app}.efi`.

### hello.efi

Print hello world.

### netboot.efi

Application to boot HTTP images (typically UKI or ISO).  Accepts an
URL or a shortcut on the command line, goes update boot configuration
to load this on next boot, resets the machine.

Known shortcuts:
 * coreos - load latest coreos release
 * xyx - boot from http://netboot.xyz

### fwcfg.efi

Print some qemu fwcfg info.

### pciscan.efi

Scan and list pci devices.

### debuglog.efi

Print OVMF memory debug log.  Needs edk2-stable202508 or newer, and
the `DEBUG_TO_MEM` OVMF build option being enabled.
