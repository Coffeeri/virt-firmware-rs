
## bios images for testing

The images are not an actual bios.  They are binaries running tests
wrapped as firmware image.  They are indended to be used with qemu.
They can be used via `qemu-system-x86_64 -nographic -bios
${image}.bin`.

The startup code (`entry32.S`) will first switch to 32-bit mode, then
call into C code.

### hello

Print hello world.

### info

Discover and print some system information.

### vmfwupdate

Dump vmfwupdate fwcfg files.
