
## efi variable store implementation in rust

Project goals:

 * Provide a efi variable store implementation for SVSM.
 * Replace the current qemu variable store implementation.

What is there:

 * basic variable store implementation (store.rs).
 * support for variable locking and variable policies (policy.rs).
 * partial support for authenticated variables + secure boot (authvar.rs).
 * parser for json variable store format used by qemu (json.rs).
 * parser for EFI_MM_COMMUNICATE protocol buffers (mm/).
 * code to parse and replay EFI_MM_COMMUNICATE protocol traces (pcap/).
 * test cases (tests/).
 * test utilities (src/bin/).

What is needed still:

 * more test cases.
 * full support for authenticated variables (needs pkcs7 signature verification).
 * support for persistent variables in svsm (needs svsm storage support).
