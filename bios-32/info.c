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
            putsln(" memory");
            break;
        case 2:
            putsln(" reserved");
            break;
        default:
            putsln(" unknown");
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
    putsln("");
}

void print_romlayout(void)
{
    putsln("romlayout");

    print_range("    code    ", &copy_start, &data_start);
    print_range("    data    ", &data_start, &copy_end);
    print_range("    stack   ", &stack_start, &stack_end);
    print_range("    flash   ", &copy_from, &flash_end);
}

__attribute__((regparm(1)))
void main(uint32_t boot_mode)
{
    serial_init();

    puts("boot mode: ");
    putl(boot_mode);
    putsln("\r\n---");

    print_romlayout();
    putsln("---");
    pci_init_info(true);
    putsln("---");

    if (fwcfg_detect()) {
        putsln("---");
        fwcfg_readdir(NULL, NULL, NULL);
        putsln("---");
        fwcfg_print_e820();
        putsln("---");
        fwcfg_hexdump("etc/e820", 128);
    }

    acpi_pm_poweroff();
}
