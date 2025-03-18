#include "lib.h"

void fwcfg_print_e820(void)
{
    uint16_t item = 0;
    uint32_t size;
    uint32_t type;

    fwcfg_readdir("etc/e820", &item, &size);
    if (!item)
        return;

    fwcfg_select(item);
    for (uint32_t i = 0; i < size; i += 20) {
        puts("e820: base ");
        putq(fwcfg_read_le64());
        puts(" size ");
        putq(fwcfg_read_le64());
        type = fwcfg_read_le32();
        switch (type) {
        case 1:
            puts(" memory\n");
            break;
        case 2:
            puts(" reserved\n");
            break;
        default:
            puts(" unknown\n");
            break;
        }
    }
}

void print_addr(const char *prefix, void *ptr)
{
    puts(prefix);
    puts(": ");
    putl((uint32_t)ptr);
    puts("\n");
}

void print_romlayout(void)
{
    puts("romlayout\n");

    puts("  32-bit code\n");
    print_addr("    .text   ", &section_text);
    puts("  data\n");
    print_addr("    .data   ", &section_data);
    print_addr("    .rodata ", &section_rodata);
    puts("  unused space\n");
    print_addr("     <free> ", &section_end);
    puts("  16-bit (at fixed addrs)\n");
    print_addr("    .gdt    ", &section_gdt);
    print_addr("    .gdtd   ", &section_gdtd);
    print_addr("    .text16 ", &section_text16);
    print_addr("    .entry16", &section_entry16);
}

void main(void)
{
    serial_init();
    print_romlayout();
    puts("---\n");
    pci_init_info(true);

    if (fwcfg_detect()) {
        puts("---\n");
        fwcfg_readdir(NULL, NULL, NULL);
        puts("---\n");
        fwcfg_print_e820();
        puts("---\n");
        fwcfg_hexdump("etc/e820", 128);
    }

    acpi_pm_poweroff();
}
