SECTIONS
{
    /* 64-bit code */
    . = 0xffff0000;
    section_text = .;
    .text : {
        *(.text)
        *(.text.*)
    }

    /* 32-bit code */
    . = ALIGN(4096);
    section_text32 = .;
    .text32 : {
        KEEP(*(.text32))
    }

    /* global offsets */
    . = ALIGN(4096);
    section_got = .;
    .got : {
        *(.got)
    }

    /* data */
    . = ALIGN(4096);
    section_data = .;
    .data : {
        *(.data)
        *(.data.*)
    }
    .bss : {
        *(.bss)
        *(.bss.*)
    }

    . = ALIGN(4096);
    section_rodata = .;
    .rodata : {
        *(.rodata)
        *(.rodata.*)
    }

    /* page tables */
    . = ALIGN(4096);
    section_pgtable = .;
    .pgtable : {
        KEEP(*(.pgtable))
    }

    . = ALIGN(4096);
    section_end = .;

    /* GDT */
    . = 0xffffff00;
    section_gdt = .;
    .gdt : {
        KEEP(*(.gdt))
    }

    /* GDT descriptor */
    . = 0xffffff70;
    section_gdtd = .;
    .gdtd : {
        KEEP(*(.gdtd))
    }

    /* 16-bit code */
    . = 0xffffff80;
    section_text16 = .;
    .text16 : {
        KEEP(*(.text16))
    }

    /* 16-bit cpu reset entry point */
    . = 0xfffffff0;
    section_entry16 = .;
    .entry16 : {
        KEEP(*(.entry))
    }

    /* Discard unnecessary sections */
    /DISCARD/ : {
        *(.note*)
        *(.eh_frame*)
    }
}

ENTRY(entry16)
