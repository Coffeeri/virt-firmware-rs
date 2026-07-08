#include "lib.h"

void main(void)
{
    serial_init();

    puts("\n");
    puts("Hello World\n");
    puts("\n");

    pci_init_info(false);
    acpi_pm_sleep(1);

    puts("reset ...\n");
    outb(0xcf9, 0x0f);
}
