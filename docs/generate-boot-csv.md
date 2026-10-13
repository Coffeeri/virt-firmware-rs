# Command-Line Help for `generate-boot-csv`

This document contains the help content for the `generate-boot-csv` command-line program.

**Command Overview:**

* [`generate-boot-csv`↴](#generate-boot-csv)

## `generate-boot-csv`



Generate BOOT.CSV file for fallback.efi with entries for all UKIs
found in the $ESP/EFI/Linux.  Additional EFI binaries can be passed
via --add command line switch.

This utility intentionally does not try to autodetect mount points or
read EFI variables.  So it can safely be used in environments like the
anaconda installer (kickstart scripts) or image builders.  This
implies the utility can not autodetect where your ESP is, so you have
to use the --esp switch if your ESP is not mounted at /efi.



**Usage:** `generate-boot-csv [OPTIONS]`

###### **Options:**

* `-l`, `--loglevel <LEVEL>` — set loglevel

  Default value: `info`
* `--esp <ESP>` — set efi system partition location
* `--add <EFI>` — add efi binary
* `--csv <CSV>` — override csv output location
* `--cmdline <APPEND>` — override kernel cmd line (default: /etc/kernel/cmdline)



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
