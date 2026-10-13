# Command-Line Help for `check-sb-signature`

This document contains the help content for the `check-sb-signature` command-line program.

**Command Overview:**

* [`check-sb-signature`↴](#check-sb-signature)

## `check-sb-signature`

pkcs7 signature checker for efi binaries and efi variable updates

**Usage:** `check-sb-signature [OPTIONS]`

###### **Options:**

* `-l`, `--loglevel <LEVEL>` — set loglevel

  Default value: `info`
* `--pk-enroll <OPT>` — select 'PK' option

  Possible values:
  - `mgmt`:
    EFI_CERT_EXTERNAL_MANAGEMENT_GUID
  - `redhat`:
    Red Hat Secure Boot (PK/KEK key 1)
  - `microsoft`:
    Windows OEM Devices PK

* `--default-kek` — use default 'KEK' certificates
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

* `--cert <FILE>` — load additional certificates
* `--pe <FILE>` [alias: `efi`] — pe/efi binary to inspect
* `--kek <FILE>` — KEK variable update to inspect
* `--db <FILE>` — db variable update to inspect
* `--dbx <FILE>` — dbx variable update to inspect



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
