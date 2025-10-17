ROM_SIZE = 128k;

MEMORY
{
    lowram   : ORIGIN = 0, LENGTH = 640k
    highram  : ORIGIN = 1M, LENGTH = 15M
    flash    : ORIGIN = (4096M - ROM_SIZE), LENGTH = ROM_SIZE
}

SECTIONS
{
    /* 64-bit code */
    .text 2M : ALIGN(4096) {
        copy_start = .;
        *(.text)
        *(.text.*)
    } > highram AT > flash

    copy_from = LOADADDR(.text);

    /* exception handler stubs */
    .stub : ALIGN(4096) {
        KEEP(*(.stub))
    } > highram AT > flash

    /* r/o data */
    .rodata : ALIGN(4096) {
        *(.rodata)
        *(.rodata.*)
    } > highram AT > flash

    /* global offsets */
    .got : ALIGN(4096) {
        *(.got)
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
        copy_end = .;
    } > highram AT > flash

    /* stack */
    .stack (NOLOAD) : ALIGN(2M) {
        stack_start = .;
        . += 2M;
        stack_end = .;
    } > highram

    /* heap */
    .heap (NOLOAD) : ALIGN(2M) {
        heap_start = .;
        . += 2M;
        heap_end = .;
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
        KEEP(*(.entry))
    } > flash

    /* Discard unnecessary sections */
    /DISCARD/ : {
        *(.note*)
        *(.eh_frame*)
    }
}

ENTRY(entry16)
