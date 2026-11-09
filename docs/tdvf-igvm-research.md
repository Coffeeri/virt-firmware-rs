# TDVF requirements for an IGVM wrapper

This note records the pre-implementation assessment. Its proposed approach
is superseded by the embedded metadata contract in
[TDVF boot interface](tdvf-boot.md). That document describes current behavior
and remaining loader integration.

Research date: 2026-11-09. Scope: packaging an existing TDVF image and the
corresponding loader contract. This does not assess implementing TDX inside
this repository's firmware.

## Binary metadata

Intel's guide specifies a version-1 `TDVF` descriptor: a 16-byte header followed
by 32-byte section descriptors. Each section describes its file offset, raw
length, guest address, memory length, type, and attributes. Memory addresses
and lengths are 4 KiB aligned. The loader zero-fills memory beyond raw data.
The guide's legacy locator is a four-byte image-relative descriptor offset at
image end minus `0x20`.

The section contract is:

| Type | Meaning | Host action | Measurement |
| --- | --- | --- | --- |
| 0 | BFV | Import private pages | MRTD |
| 1 | CFV | Import private pages | Firmware extends RTMR[0] |
| 2 | TD_HOB | Populate and import host information | Firmware extends RTMR[0] |
| 3 | TempMem | Import zero pages | No content extension |
| 4 | PermMem | Supply unaccepted memory | Guest accepts |
| 5 | Payload | Import payload | MRTD or RTMR[1] |
| 6 | PayloadParam | Import parameters | RTMR[1] |
| 7 | TD_INFO | Describe information within BFV | No separate allocation |

Attribute bit 0 controls `MR.EXTEND`. Bit 1 requests `PAGE.AUG` rather than
`PAGE.ADD`. A valid image needs a BFV containing the reset vector. TDVF starts
in 32-bit protected mode with paging disabled.

Source: [Intel TDVF Design Guide rev. 004, sections 4.1.1 and 11.1-11.2,
printed pages 21 and 51-54](https://cdrdv2-public.intel.com/733585/tdx-virtual-firmware-design-guide-rev-004-20231206.pdf).

## Current EDK II and QEMU locator

Current EDK II publishes TDX metadata in the OVMF GUID footer table. The entry
GUID is `e47a6535-984a-4798-865e-4685a7bf8ec2`. Its four-byte value is a
backward offset from the image end to the descriptor, unlike the legacy
locator above. The footer itself is found using GUID
`96b582de-1fb2-45f7-baea-a366c55a082d` at image end minus `0x30`. Entries
include their own lengths and are traversed backward.

Source: [EDK II ResetVectorVtf0.nasm.inc](https://github.com/tianocore/edk2/blob/master/OvmfPkg/ResetVector/Ia16/ResetVectorVtf0.nasm.inc).

QEMU's parser uses this GUID-table locator. It checks the signature, version,
descriptor length, section count, page alignment, and memory length versus
raw length. Its current implementation accepts only BFV, CFV, TD_HOB, and
TempMem. BFV/CFV require nonzero raw length. TD_HOB/TempMem require zero raw
length. The Intel guide describes a broader format than this particular
consumer implements.

Source: [QEMU hw/i386/tdvf.c](https://github.com/qemu/qemu/blob/master/hw/i386/tdvf.c).

## HOB contract

The PI specification defines a HOB list beginning with the Phase Handoff
Information Table (PHIT) and ending with an end-of-list HOB. Every HOB has a
generic header. HOB lengths and addresses are multiples of eight bytes.
The list becomes read-only after handoff. PI's normal DXE handoff also needs
CPU, memory, and firmware-volume information. A host TD HOB and the firmware's
eventual DXE HOB serve different handoffs.

Source: [PI Specification 1.6, Volume 3 sections 4.4-4.5 and 5, printed pages
124 onward, and Volume 2 section 9.6.1](https://uefi.org/sites/default/files/resources/PI_Spec_1_6.pdf#G8.1008478).

QEMU creates the TD HOB inside the address and size declared by the image's
TD_HOB descriptor. It writes a PHIT with zero memory/free-memory bounds,
resource descriptors for RAM ranges, and an end marker. It distinguishes
already-added private system memory from unaccepted memory using resource
types and attributes. This is generated from the actual VM memory topology,
not just the firmware bytes.

Source: [QEMU hw/i386/tdvf-hob.c](https://github.com/qemu/qemu/blob/master/hw/i386/tdvf-hob.c).

QEMU's current declarations use resource type `7` for unaccepted memory.
The implementation needs these TDX-relevant definitions in addition to the
generic HOB container layout.

Source: [QEMU UEFI definitions](https://github.com/qemu/qemu/blob/master/include/standard-headers/uefi/uefi.h).

## Loader and boot responsibilities

QEMU's TDX loader allocates the HOB and temporary regions, adjusts its RAM
classification, constructs the HOB, initializes every vCPU with its GPA,
imports firmware regions, and finalizes the TD. Its import loop follows the
metadata entry order. For each region, `MR_EXTEND` selects
`KVM_TDX_MEASURE_MEMORY_REGION` when invoking `KVM_TDX_INIT_MEM_REGION`.
The loader must perform these operations before private memory becomes
inaccessible to ordinary host writes.

Source: [QEMU target/i386/kvm/tdx.c, tdx_finalize_vm(),
tdx_post_init_vcpus(), and tdx_init_fw_mem_region()](https://github.com/qemu/qemu/blob/master/target/i386/kvm/tdx.c).

Linux documents `KVM_TDX_INIT_VCPU.data` as the initial guest RCX value.
Combined with QEMU's call above, the TD HOB GPA reaches the firmware in RCX.
The import ABI optionally measures each region and requires private memory
attributes before import. `KVM_TDX_FINALIZE_VM` completes the initial
measurement before execution.

Source: [Linux KVM TDX ABI documentation, KVM_TDX_INIT_VCPU,
KVM_TDX_INIT_MEM_REGION, and KVM_TDX_FINALIZE_VM](https://docs.kernel.org/virt/kvm/x86/intel-tdx.html).

QEMU requires a TDX-capable host, initialized TDX module, and KVM support.
TDVF runs from private RAM rather than a pflash mapping. QEMU documents
`-bios OVMF.fd` as its TDVF loading interface.

Source: [QEMU TDX documentation](https://www.qemu.org/docs/master/system/i386/tdx.html).

## Implications for implementation scope

The following are engineering conclusions from these contracts:

- A wrapper can reuse an existing TDVF implementation. Adding TDX to a
  firmware implementation is a separate, substantially larger task.
- A converter must preserve section placement, content, zero-fill, and
  measurement attributes. It should explicitly reject section types whose
  semantics it cannot represent.
- Handling `MR.EXTEND` alone does not implement `PAGE.AUG`. A converter
  limited to initial imported private pages should reject sections requesting
  augmentation until its loader contract supports them.
- A fixed empty HOB is insufficient for general boot. Either the selected
  loader must populate it from VM configuration, or an agreed parameter
  mechanism must supply equivalent information expected by that firmware.
- Compare expected MRTD with the original firmware loading path. Any reordering
  of page import directives needs deliberate validation.
- Successful IGVM serialization proves the container is structurally valid.
  Successful boot also requires agreement with the target VMM about vCPU
  startup, firmware information, memory acceptance, and platform devices.

The source links to EDK II and QEMU track their default branches. Recheck their
behavior when selecting concrete revisions for implementation.

## Historical local implementation assessment

The following assessment describes the workspace before implementation:

- [Cargo.toml](../Cargo.toml) accepts `igvm` and `igvm_defs` versions
  `>= 0.4, < 0.6`. [Cargo.lock](../Cargo.lock) selects 0.5.0 for both.
  The installed definitions include `IgvmPlatformType::TDX`,
  `IGVM_TDX_PLATFORM_VERSION`, `TdxPolicy`, and private unmeasured page flags.
  These definitions support initial TDX page conversion.
- [ovmfmeta.rs](../igvm-tools/src/ovmfmeta.rs) already locates the TDX GUID
  entry and decodes section file ranges, GPAs, sizes, and types 0 through 3.
  `OvmfRegion::new_tdx()` discards the section flags. The parser also needs
  strict signature, version, descriptor-length, file-range, alignment, and
  overlap validation before its output controls image construction. Parsing
  failures currently return `Option` and are ignored by the outer loop.
- [builder.rs](../igvm-tools/src/builder.rs) can already emit private measured
  and unmeasured pages. Its data helpers use the mask of every configured
  platform. TDX needs explicit platform masks when sharing an image with
  native or SNP directives. Its duplicate-page check enumerates only native
  and SNP bits. `finalize()` sorts page data by mask and GPA, which would
  change TDVF import order and MRTD relative to QEMU's metadata-order path.
- [igvm-wrap.rs](../igvm-tools/src/bin/igvm-wrap.rs) currently emits a whole
  firmware image below 4 GiB and an optional low-memory alias. TDX needs its
  own metadata-driven section path, with BFV/CFV copied to their declared
  addresses and the declared temporary regions initialized. It must avoid
  applying generic firmware aliases or variable-store placement to TDX.
- [libefi HOB definitions](../libefi/src/hob/core.rs) already provide generic
  headers, resource descriptors, an end marker, and unaccepted-memory type 7.
  HOB construction can extend these definitions without another dependency.
  A PHIT definition and a bounded list builder would still be needed.

The IGVM specification explicitly forbids `VP_CONTEXT` for architectures with
an architectural reset state, including TDX. The wrapper must not reuse native
or SNP initial CPU contexts. Passing the HOB GPA in initial RCX is a separate
loader responsibility.

Source: [IGVM VP context definition](https://github.com/microsoft/igvm/blob/main/igvm_defs/src/lib.rs),
also checked against the installed 0.5.0 crate source.

### Existing local converter

The sibling checkout `/home/lkohler/dev/buildigvm` already contains
`src/tdvf_firmware.rs`. Its implementation validates version-1 metadata,
preserves section order, emits section-specific GPAs and measurement flags,
zero-pads raw data, and rejects unsupported section types and augmentation.
Its conversion logic is useful reference material for this workspace.

Its README explicitly states that TD_HOB pages are zero placeholders and that
boot integration must supply the HOB and initial RCX. It demonstrates the
conversion mechanics, not a completed boot path. This is a separate binary,
not an `igvm-wrap` dependency.

The sample `/home/lkohler/dev/buildigvm/TDX-OVMF.fd` has this metadata, decoded
directly from its GUID table and descriptor:

| Order | Section | GPA | Memory bytes | MR_EXTEND |
| --- | --- | --- | --- | --- |
| 0 | BFV | `0xffc84000` | `0x37c000` | Yes |
| 1 | CFV | `0xffc00000` | `0x84000` | No |
| 2 | TempMem | `0x810000` | `0x10000` | No |
| 3 | TempMem | `0x80b000` | `0x2000` | No |
| 4 | TD_HOB | `0x809000` | `0x2000` | No |
| 5 | TempMem | `0x800000` | `0x6000` | No |

This makes the ordering and measurement differences concrete. Sorting by GPA
moves the BFV from first to last. Measuring the whole image also measures CFV
contents that this metadata does not extend into MRTD. Unmeasured private
pages still contribute their page-add operations to MRTD.

The local `/home/lkohler/dev/cyberus-cloud-hypervisor` checkout has TDX platform
selection and an initial-RCX configuration field. Its current
`vmm/src/igvm/igvm_loader.rs` still asserts an SNP platform and configures KVM
page types through SNP constants. Those pieces do not establish a working
TDVF IGVM import path. Loader integration remains separate work.

QEMU's current published IGVM documentation lists native and AMD SEV-family
support, without TDX. Its raw TDVF loader is a reference implementation, but
it should not be assumed to load the proposed TDX IGVM output unchanged.

Source: [QEMU IGVM supported platforms](https://www.qemu.org/docs/master/system/igvm.html#supported-platforms).

### Suggested scope and effort

These are engineering estimates, not measured implementation times:

| Deliverable | Estimated effort | Assumptions |
| --- | --- | --- |
| Validated TDX-only TDVF-to-IGVM conversion | 2-4 engineer days | Existing TDVF, version 1, types 0-3, no PAGE_AUG, focused tests |
| Boot and measurement validation on one VMM | About 1-2 weeks total | TDX host available and an existing VMM TDX launch implementation |
| Portable runtime HOB integration across VMMs | Separate design and integration effort | Requires agreement on runtime input and startup conventions |

A practical first implementation would support one existing TDVF build and
one loader. The HOB can be supplied as a validated prebuilt blob for an exact
VM layout, or generated by the loader from actual RAM and acceptance state.
The loader must populate unmeasured HOB pages before importing them into the
TD, and initialize RCX with their GPA. Existing IGVM memory-map parameters
use a different layout from PI HOBs. They cannot replace the HOB bytes without
a loader or firmware translation step.

After parser and conversion tests, acceptance should include a real boot,
checking expected MRTD against the chosen loading sequence, and confirming
native/SNP behavior still works. This assessment did not perform a TDX boot.
