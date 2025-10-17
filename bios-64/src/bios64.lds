ROM_SIZE = 64k;

MEMORY
{
    lowram   : ORIGIN = 0, LENGTH = 640k
    highram  : ORIGIN = 1M, LENGTH = 15M
    flash    : ORIGIN = (4096M - ROM_SIZE), LENGTH = ROM_SIZE
}

SECTIONS
{
    /* 64-bit code */
    .text : ALIGN(4096) {
        *(.text)
        *(.text.*)
    } > flash

    /* 32-bit code */
    .text32 : ALIGN(4096) {
        KEEP(*(.text32))
    } > flash

    /* global offsets */
    .got : ALIGN(4096) {
        *(.got)
    } > flash

    /* data */
    .data : ALIGN(4096) {
        *(.data)
        *(.data.*)
    } > flash

    .bss : ALIGN(4096) {
        *(.bss)
        *(.bss.*)
    } > flash

    .rodata : ALIGN(4096) {
        *(.rodata)
        *(.rodata.*)
    } > flash

    /* page tables */
    .pgtable : ALIGN(4096) {
        KEEP(*(.pgtable))
    } > flash

    /* exception handler stubs */
    .stub 0xffffe000 : {
        KEEP(*(.stub))
    } > flash

    /* IDT */
    .idt 0xfffffc00 : {
        KEEP(*(.idt))
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
