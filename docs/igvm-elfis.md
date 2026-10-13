# Command-Line Help for `igvm-elfis`

This document contains the help content for the `igvm-elfis` command-line program.

**Command Overview:**

* [`igvm-elfis`↴](#igvm-elfis)

## `igvm-elfis`

convert elf images into igvm

**Usage:** `igvm-elfis [OPTIONS] --input <ELF>`

###### **Options:**

* `--real16` — boot firmware in real mode (default)
* `--flat32` — boot firmware in protected mode
* `--regions` — list regions in elf image
* `--inspect` — inspect generated igvm
* `-i`, `--input <ELF>` — input elf image
* `-k`, `--kernel <EFI>` — input efi binary (usually linux kernel)
* `-o`, `--output <IGVM>` — output igvm image



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
