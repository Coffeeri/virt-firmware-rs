# Command-Line Help for `igvm-update`

This document contains the help content for the `igvm-update` command-line program.

**Command Overview:**

* [`igvm-update`↴](#igvm-update)

## `igvm-update`

add kernel to igvm file

**Usage:** `igvm-update [OPTIONS] --input <FILE>`

###### **Options:**

* `-i`, `--input <FILE>` — input igvm image
* `-s`, `--shim <SHIM>` — input shim image (optional)
* `-k`, `--kernel <KERNEL>` — input kernel image (can be any efi image)
* `--add-hash-sha256` [alias: `add-hash`] — add authenticode sha256 hashes for shim and kernel to 'db'
* `--add-cert-ms` — add microsoft certificates to 'db'
* `--profile <NAME>` — select 'db' profile

  Possible values:
  - `none`:
    no certificates
  - `win11`:
    Windows CA certificates, 2011 + 2023
  - `win23`:
    Windows CA certificate, 2023 only
  - `win+rom23`:
    Windows CA + Option ROM certificates, 2023 only
  - `uefi11`:
    UEFI CA certificates, 2011 + 2023
  - `uefi23`:
    UEFI CA certificate, 2023
  - `uefi+rom23`:
    UEFI CA + Option ROM certificates, 2023
  - `redhat`:
    Red Hat UEFI CA certificate
  - `redhat+rom`:
    Red Hat UEFI CA + Option ROM certificates
  - `all`:
    Both Windows and UEFI CA certificates, 2011 + 2023

* `--enroll <MODE>` — select 'PK' enroll mode

  Default value: `redhat`

  Possible values:
  - `mgmt`:
    EFI_CERT_EXTERNAL_MANAGEMENT_GUID
  - `redhat`:
    Red Hat Secure Boot (PK/KEK key 1)
  - `microsoft`:
    Windows OEM Devices PK

* `--arch <ARCH>` — select efi architecture for 'dbx'

  Possible values: `aa64`, `x64`

* `--inspect` — inspect updated igvm
* `-o`, `--output <FILE>` — output igvm image



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
