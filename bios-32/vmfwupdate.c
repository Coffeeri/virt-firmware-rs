#include "lib.h"

static const char *filelist[] = {
    "etc/vmfwupdate/cap",
    "etc/vmfwupdate/disable",
    "etc/vmfwupdate/bios-size",
    "etc/vmfwupdate/fw-blob",
    "etc/vmfwupdate/opaque-blob",
    NULL
};

void main(void)
{
    serial_init();

    puts("\n");
    puts("launch ok\n");
    puts("\n");

    if (fwcfg_detect()) {
        for (uint32_t i = 0; filelist[i] != NULL; i++) {
            puts("\n");
            fwcfg_hexdump(filelist[i], 128);
        }
    }

    pci_init_info(false);
    acpi_pm_poweroff();
}
