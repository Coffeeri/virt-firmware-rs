
## efi variable store implementation in rust

Project goals:

 * Provide a efi variable store implementation for SVSM.
 * Replace the current qemu variable store implementation.

What is there:

 * basic variable store implementation (store.rs).
 * parser for json variable store format used by qemu (json.rs).
 * parser for EFI_MM_COMMUNICATE protocol buffers (mm/).
 * code to parse and replay EFI_MM_COMMUNICATE protocol traces (pcap/).
 * test cases (tests/).
 * test utilities (src/bin/).

What is needed still:

 * support for authenticated efi variables.
 * support for secure boot.
 * support for variable locking and variable policies.
 * more test cases.
 * svsm integration.
