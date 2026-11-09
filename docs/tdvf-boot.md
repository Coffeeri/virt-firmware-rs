# Experimental TDVF boot interface

`igvm-wrap --tdx` wraps an existing TDVF image for the Cloud Hypervisor
TDVF HOB convention. It emits TDX-only IGVM, preserving section import order,
private-page contents, zero fill, and the `MR_EXTEND` attribute. It supports
version-1 metadata in the OVMF GUID table, with BFV, CFV, temporary memory,
and exactly one TD HOB. Other section types, augmentation, malformed metadata,
and incompatible command-line options fail explicitly.

```sh
cargo run -p virtfw-igvm-tools --bin igvm-wrap -- \
    --tdx --input TDX-OVMF.fd --output TDX-OVMF.igvm
cargo run -p virtfw-igvm-tools --bin igvm-inspect -- TDX-OVMF.igvm
cargo test -p virtfw-igvm-tools --test tdvf_wrap -- --include-ignored
```

The last command additionally tests the local, untracked `TDX-OVMF.fd`.
The normal suite uses synthetic fixtures and needs no firmware download.

## Embedded metadata and HOB binding

The image uses standard IGVM directives and the upstream `igvm` and
`igvm_defs` 0.5 crates. The BFV retains its original bytes, including TDVF
metadata. A standard `ParameterArea` and `ParameterInsert` reserve and
import the runtime TD HOB buffer at the metadata-declared GPA.

The loader must explicitly select the Cloud Hypervisor TDVF profile. It finds
the OVMF footer GUID in the architectural x86 reset window ending at 4 GiB,
then decodes the TDVF descriptor. This profile requires a BFV raw section
ending at 4 GiB that contains both the footer and descriptor. Missing or
ambiguous metadata fails validation.

The loader validates the full ordered import plan against the descriptor:
GPAs, sizes, flags, zero fill, and the exact HOB binding must match.
The converter copies BFV contents unchanged. It resolves the HOB parameter area by its GPA and size, not by its
index or by choosing the first unmeasured area. The converter currently uses
area index 0. The HOB insertion occupies the original TD HOB section's
position among page imports. Even unmeasured pages contribute page-add
records to MRTD. There is no TDX `VP_CONTEXT`.

## Runtime preparation

`tdvf_launch::prepare_tdvf_launch()` validates the embedded metadata and HOB
binding, then constructs an ordered private-page import list plus
`initial_rcx`. The HOB GPA supplies the initial RCX value through
`KVM_TDX_INIT_VCPU`.
The caller supplies actual VM RAM/reservations and generated ACPI tables.
Initial RAM is unaccepted. Preparation derives its accepted subranges from
the actual import plan, including zero pages and HOB pages.

The HOB serializer emits a leading PHIT, resource descriptors, ACPI GUID
HOBs using `gUefiOvmfPkgTdxAcpiHobGuid`, and an end marker. The PHIT memory
bounds are zero. ACPI tables must have valid lengths and checksums and include
exactly one DSDT. Complete serialized output must fit the declared capacity.
For the supplied sample, that capacity is only 8 KiB.

The VMM must initialize the TD and vCPUs, set private memory attributes,
import the returned pages in order with the indicated measurement flag,
finalize the TD, and then run it. Preparation performs no KVM operations.
It must complete before importing private pages. IGVM policy fields require
explicit translation to hardware attributes and capability checks.

## Compatibility and limits

A standard IGVM parser can read the image. Boot requires a TDVF-aware loader
that selects and implements the Cloud Hypervisor TDVF profile. An arbitrary
IGVM loader cannot boot the image unchanged.

The sibling Cloud Hypervisor VMM has not been wired to this interface. Its
KVM primitives alone do not implement the required launch lifecycle. These
images are not claimed to boot with an unchanged Cloud Hypervisor or QEMU.
No hardware boot or attestation result is implied by serialization tests.

Pin the firmware build and platform profile when integrating a loader.
Firmware runtime requirements and ACPI delivery conventions can differ.

## Validation

The supplied sample has SHA-256
`05b5e640a9c67306ed4b1f0748b1718c3ae9add00a7edd085dd939caf940073f`.
Its six sections produce 1,050 imported pages, including two runtime HOB
pages. Of those, 892 pages have content extended into MRTD. The independent
software transcript comparison produces:

```text
7483e830510143e18202a3dd29fd7bc3746eeb55848f90e0a53238016ad85f84b27f0dfb6241c6f74100924e0d2a44fd
```

This is a software prediction, not a measurement obtained from TDX hardware.
The sample test independently decodes raw metadata and compares its transcript
with the serialized IGVM imports. Synthetic tests cover parser failures,
option conflicts, native wrapping, import overlap, HOB bounds, ACPI validation,
runtime classification, import order, metadata, and HOB binding validation.
The original installed `igvm-inspect` also parses the regenerated image.

Full-workspace `cargo clippy --all --all-targets` cannot use a single host
target: `efi-apps` requires a UEFI target and `mefisto` requires
`x86_64-unknown-none`.

References:

- [Intel TDVF design guide, sections 4 and 11](https://cdrdv2-public.intel.com/733585/tdx-virtual-firmware-design-guide-rev-004-20231206.pdf)
- [IGVM definitions](https://github.com/microsoft/igvm/blob/affe163c949c744f88f6d5831bcc4b3c78c4dc1d/igvm_defs/src/lib.rs)
- [Linux KVM TDX ABI](https://docs.kernel.org/virt/kvm/x86/intel-tdx.html)
- [EDK II Cloud Hypervisor ACPI handoff](https://github.com/tianocore/edk2/blob/master/OvmfPkg/AcpiPlatformDxe/CloudHvAcpi.c)
