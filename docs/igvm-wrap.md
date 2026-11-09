# Command-Line Help for `igvm-wrap`

This document contains the help content for the `igvm-wrap` command-line program.

**Command Overview:**

* [`igvm-wrap`↴](#igvm-wrap)

## `igvm-wrap`

wrap classic native firmware image as igvm

**Usage:** `igvm-wrap [OPTIONS]`

###### **Options:**

* `--tdx` — wrap TDVF for a TDVF-aware TDX loader
* `--native` — enable native platform (default)
* `--no-native` — disable native platform
* `--snp` — enable sev-snp platform
* `--no-snp` — disable sev-snp platform (default)
* `--params` — generate parameters (default)
* `--no-params` — do not generate parameters
* `--real16` — boot firmware in real mode (default)
* `--flat32` — boot firmware in protected mode
* `--meta` — print ovmf metadata
* `--inspect` — inspect generated igvm
* `-i`, `--input <CODE>` — input firmware image
* `--vars <VARS>` — input uefi vars image
* `-o`, `--output <IGVM>` — output igvm image



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
