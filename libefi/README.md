
## library to read + write efi data structures

Implemented mostly what is needed to manage efi boot configuration,
i.e. BootNNNN. BootOrder, etc variables.

Some tooling needed to manage efi boot configuration for linux is
included too, such as functions to read/write efi variables via
efivarfs.
