#include "lib.h"

void main(void)
{
    serial_init();

    putsln("");
    putsln("Hello World");
    putsln("");

    pci_init_info(false);
    acpi_pm_sleep(1);

    putsln("poweroff ...");
    acpi_pm_poweroff();
}
