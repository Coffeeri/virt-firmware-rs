
## bios images for testing

The images are not an actual bios.  They are binaries running tests
wrapped as firmware image.  They are indended to be used with qemu.
They can be used via `qemu-system-x86_64 -nographic -bios
${image}.bin`.

The startup code (`entry64.S`) will first switch to 32-bit mode, then
to long mode, finally call into rust code.

### mini

Print hello world to serial console, halt.
