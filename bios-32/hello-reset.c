#include "lib.h"

void main(void)
{
    serial_init();

    putsln("");
    putsln("Hello World");
    putsln("");

    pci_init_info(false);
    acpi_pm_sleep(1);

    putsln("reset ...");
    outb(0xcf9, 0x0f);
}
