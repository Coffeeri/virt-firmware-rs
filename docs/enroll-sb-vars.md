# Command-Line Help for `enroll-sb-vars`

This document contains the help content for the `enroll-sb-vars` command-line program.

**Command Overview:**

* [`enroll-sb-vars`↴](#enroll-sb-vars)

## `enroll-sb-vars`

Enroll databases and enable secure boot.

Enroll secure boot certificates.  By default the utility will look at
your boot loader (usually shim.efi) and enroll the CA certificate used
to sign the boot loader.  In case the boot loader has multiple
signatures you might have to pick one using '--ca-subject'.
Alternatively you can pick a set of certificates using '--profile'.
Additional certificates can be loaded from files using '--db-cert'.
You can use dry run mode to verify the utility will do what you want
before actually enrolling the certificates.

This needs a recent UEFI firmware version which accepts an unsigned
platform key (PK).  The config switch (PcdRequireSelfSignedPk) was
added to edk2 in 2023.  The default for OVMF has been changed in
upstream edk2 in 2026 (edk2-stable202605 or newer).  When using the
qemu variable store (aka '-device uefi-vars-x64') you need qemu 11.2
or newer.

This utility is restricted to virtual machines.



**Usage:** `enroll-sb-vars [OPTIONS]`

###### **Options:**

* `-l`, `--loglevel <LEVEL>` — set loglevel

  Default value: `info`
* `--dry-run` — dry run mode

   List the certificates which would be enrolled without actually updating the UEFI secure boot configuration.
* `--enroll <MODE>` — select 'PK' enroll mode

  Default value: `redhat`

  Possible values:
  - `mgmt`:
    EFI_CERT_EXTERNAL_MANAGEMENT_GUID
  - `redhat`:
    Red Hat Secure Boot (PK/KEK key 1)
  - `microsoft`:
    Windows OEM Devices PK

* `-i`, `--interactive` — enable interactive 'db' certificate selection
* `--ca-subject <CN>` — select 'db' boot loader certificate

   Might be needed if your boot loader has multiple signatures to specify which signature CA should be used.
* `--profile <NAME>` — select 'db' certificate profile

   For linux systems you usually need 'uefi11' or 'uefi23'.

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

* `--db-cert <FILE>` — add certificate to 'db'

   Can be specified multiple times.  Certificates will be enrolled in addition to the other certificates.  Use '--profile none' if you want specify all certificates explicitly.



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
