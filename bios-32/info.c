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

void print_range(const char *prefix, void *from, void *to)
{
    puts(prefix);
    puts(": ");
    putl((uint32_t)from);
    puts(" - ");
    putl(((uint32_t)to)-1);
    puts("\n");
}

void print_romlayout(void)
{
    puts("romlayout\n");

    print_range("    code    ", &copy_start, &data_start);
    print_range("    data    ", &data_start, &copy_end);
    print_range("    stack   ", &stack_start, &stack_end);
    print_range("    flash   ", &copy_from, &flash_end);
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
