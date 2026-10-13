
## virt-firmware tools, rust edition

A collection of tools to deal with efi and virtual machine firmware,
written mostly in rust.  Linux tools.  EFI apps.  Firmware images for
testing purposes.

Related python project: https://gitlab.com/kraxel/virt-firmware

### rust crates

 * [efi applications](efi-apps/)
 * [efi related linux tools](efi-tools/)
 * [igvm related linux tools](igvm-tools/)
 * [helper functions for direct hardware access](libhw/)
 * [helper functions for efi variables + data structures](libefi/)
 * [uefi variable store](varstore/)
 * [64-bit firmware images](mefisto/)

### C code

 * [32-bit firmware test images](bios-32/)

### Firmware and EFI binaries

 * https://kraxel.gitlab.io/virt-firmware-rs/

### Manual pages for tools

 * [docs](docs/)

### compiling the project

Just use `make`.

### contributing

I take MRs.

There is an AI review bot active on the repo which will comment on
MRs.  It is good at spell checking.  Sometimes it offers useful
suggestions or catches inconsistencies in the patches.  Sometimes it
asks for stupid stuff like careful error checking in test cases.
Sometimes it suggests to check for error conditions which can not
happen, but the bot is apparently not clever enough to see that.
Sometimes it suggests code reorganizations where it is more a matter
of taste or personal preference whenever you take the one or the other
way.

So, don't take the bot too serious.
