
## efi variable store implementation in rust

Project goals:

 * Provide a efi variable store implementation for SVSM.
 * Replace the current qemu variable store implementation.

What is there:

 * core variable store implementation.
 * support for variable locking and variable policies.
 * partial support for authenticated variables + secure boot.
 * parser for EFI_MM_COMMUNICATE protocol buffers.
 * parser for json variable store format used by qemu.
 * code to parse and replay EFI_MM_COMMUNICATE protocol traces.
 * test cases and test utilities.

What is needed still:

 * full support for authenticated variables (needs pkcs7 signature verification).
 * support for persistent variables in svsm (needs svsm storage support).
 * better documentation.
 * more test cases.

## Features

- `std`: Enables support for the standard library.  Used for test
  cases.

- `json`: Enables read/write variables from/to JSON format, using
  serde.  Used for test cases.

- `sbtest`: Enable support for booting test kernel builds in secure
  boot mode (include `Red Hat Test Certifying CA` in `db`).
