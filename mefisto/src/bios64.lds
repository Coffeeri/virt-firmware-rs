ROM_SIZE = 2M;
HEAP_SIZE = 1M;
STACK_SIZE = 1M;

MEMORY
{
    lowram   : ORIGIN = 0, LENGTH = 640k
    highram  : ORIGIN = 1M, LENGTH = 15M
    flash    : ORIGIN = (4096M - ROM_SIZE), LENGTH = ROM_SIZE
}

SECTIONS
{
    /* 64-bit code */
    .text 8M : ALIGN(4096) {
        copy_start = .;
        *(.text)
        *(.text.*)
    } > highram AT > flash

    copy_from = LOADADDR(.text);

    /* exception handler stubs */
    .stub : ALIGN(4096) {
        KEEP(*(.stub))
    } > highram AT > flash

    /* global offsets */
    .got : ALIGN(4096) {
        *(.got)
        *(.got.*)
    } > highram AT > flash

    /* r/o data */
    .rodata : ALIGN(4096) {
        data_start = .;
        *(.rodata)
        *(.rodata.*)
    } > highram AT > flash

    /* page tables */
    .pgtable : ALIGN(4096) {
        KEEP(*(.pgtable))
    } > highram AT > flash

    /* data */
    .data : ALIGN(4096) {
        *(.data)
        *(.data.*)
    } > highram AT > flash

    .bss : ALIGN(4096) {
        *(.bss)
        *(.bss.*)
        . = ALIGN(4096);
        copy_end = .;
    } > highram AT > flash

    /* snp special pages*/
    .snp.cpuid (NOLOAD) : ALIGN(4096) {
        *(.snp.cpuid)
    } > highram
    .snp.secrets (NOLOAD) : ALIGN(4096) {
        *(.snp.secrets)
    } > highram

    /* igvm parameters */
    .igvm.memmap (NOLOAD) : ALIGN(4096) {
        *(.igvm.memmap)
    } > highram

    /* igvm data hobs */
    .igvm.datahobs (NOLOAD) : ALIGN(4096) {
        *(.igvm.datahobs)
    } > highram

    /* heap */
    .heap (NOLOAD) : ALIGN(2M) {
        heap_start = .;
        . += HEAP_SIZE;
        heap_end = .;
    } > highram

    /* stack */
    .stack (NOLOAD) : ALIGN(4096) {
        stack_start = .;
        . += STACK_SIZE;
        stack_end = .;
    } > highram

    /* efi memory pools */
    .pools (NOLOAD) : ALIGN(4096) {
        *(.pool.rt)
        *(.pool.bs)
    } > highram

    /* 32-bit code */
    .text32 : ALIGN(4096) {
        KEEP(*(.text32))
    } > flash

    /* GDT */
    .gdt 0xffffff00 : {
        KEEP(*(.gdt))
    } > flash

    /* GDT descriptor */
    .gdtd 0xffffff70 : {
        KEEP(*(.gdtd))
    } > flash

    /* 16-bit code */
    .text16 0xffffff80 : {
        KEEP(*(.text16))
    } > flash

    /* 16-bit cpu reset entry point */
    .entry16 0xfffffff0 : {
        KEEP(*(.entry16))
        flash_end = .;
    } > flash

    /* Discard unnecessary sections */
    /DISCARD/ : {
        *(.note*)
        *(.eh_frame*)
    }
}

ENTRY(entry16)
