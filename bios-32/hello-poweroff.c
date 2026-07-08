#include "lib.h"

void main(void)
{
    serial_init();

    puts("\n");
    puts("Hello World\n");
    puts("\n");

    pci_init_info(false);
    acpi_pm_sleep(1);
    acpi_pm_poweroff();
}
