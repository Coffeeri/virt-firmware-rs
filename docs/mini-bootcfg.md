# Command-Line Help for `mini-bootcfg`

This document contains the help content for the `mini-bootcfg` command-line program.

**Command Overview:**

* [`mini-bootcfg`↴](#mini-bootcfg)

## `mini-bootcfg`

show and manage uefi boot entries

**Usage:** `mini-bootcfg [OPTIONS]`

###### **Options:**

* `-l`, `--loglevel <LEVEL>` — set loglevel

  Default value: `info`
* `-s`, `--show` — print boot configuration
* `-v`, `--verbose` — more verbose output
* `--add-uki <FILE>` — add boot entry for UKI image FILE
* `--remove-uki <FILE>` — remove boot entry for UKI image FILE
* `--cleanup-uki` — remove stale UKI entries
* `--cmdline <CMDLINE>` — override cmd line for boot entry
* `--boot-ok` [alias: `boot-successful`] — boot is successful
* `--update-csv` — update BOOT.CSV
* `--add-uri <URI>` — add boot entry to netboot URI
* `--remove-entry <NNNN>` — remove boot entry NNNN
* `--boot-next` [alias: `once`] — boot added entry once (via BootNext)
* `--boot-order <POS>` — place added entry at POS to BootOrder
* `--dry-run` — do not actually update the configuration
* `--title <TITLE>` — set title for boot entry
* `--shim <FILE>` — use shim binary FILE
* `--print-loader` — print boot loader name
* `--print-stub-info` — print efi stub info
* `--print-stub-image` — print efi stub image



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
