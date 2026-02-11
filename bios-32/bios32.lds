ROM_SIZE = 64k;
STACK_SIZE = 1M;

MEMORY
{
    lowram   : ORIGIN = 0, LENGTH = 640k
    highram  : ORIGIN = 1M, LENGTH = 15M
    flash    : ORIGIN = (4096M - ROM_SIZE), LENGTH = ROM_SIZE
}

SECTIONS
{
    /* 32-bit code (in ram) */
    .text 8M : ALIGN(4096) {
        copy_start = .;
        *(.text)
        *(.text.*)
    } > highram AT > flash

    copy_from = LOADADDR(.text);

    /* r/o data */
    .rodata : ALIGN(4096) {
        data_start = .;
        *(.rodata)
        *(.rodata.*)
    } > highram AT > flash

    /* global offsets */
    .got : ALIGN(4096) {
        *(.got)
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

    /* stack */
    .stack (NOLOAD) : ALIGN(STACK_SIZE) {
        stack_start = .;
        . += STACK_SIZE;
        stack_end = .;
    } > highram

    /* 32-bit code (in flash) */
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
