#include "lib.h"

#define PCI_CMD_PORT    0x0cf8
#define PCI_DATA_PORT   0x0cfc

#define PCI_CFG_VENDOR    0x00
#define PCI_CFG_DEVICE    0x02

#define PCI_CMD(_bus, _slot, _func, _addr)        \
    ((_addr) |                                    \
     ((_func) << 8) |                             \
     ((_slot) << (8 + 3)) |                       \
     ((_bus) << (8 + 3 + 5)) |                    \
     (1 << 31))

#define ACPI_PM_BASE                0x600
#define ACPI_PM_CNT    (ACPI_PM_BASE + 4)

void acpi_pm_init_pc(void)
{
    // map ACPI_PM_BASE
    outl(PCI_CMD_PORT, PCI_CMD(0, 1, 3, 0x40)); // PIIX_PMBASE
    outw(PCI_DATA_PORT, ACPI_PM_BASE | 1);
    outl(PCI_CMD_PORT, PCI_CMD(0, 1, 3, 0x80)); // PIIX_PMREGMISC
    outb(PCI_DATA_PORT, 0x01);
}

void acpi_pm_init_q35(void)
{
    // map ACPI_PM_BASE
    outl(PCI_CMD_PORT, PCI_CMD(0, 0x1f, 0, 0x40)); // ICH9_LPC_PMBASE
    outl(PCI_DATA_PORT, ACPI_PM_BASE | 1);
    outl(PCI_CMD_PORT, PCI_CMD(0, 0x1f, 0, 0x44)); // ICH9_LPC_ACPI_CTRL
    outb(PCI_DATA_PORT, 0x80);
}

void acpi_pm_poweroff(void)
{
    outw(ACPI_PM_CNT, 0x2000);
}

void pci_init_info(bool verbose)
{
    uint16_t v, d;

    outl(PCI_CMD_PORT, PCI_CMD(0, 0, 0, PCI_CFG_VENDOR));
    v = inw(PCI_DATA_PORT);
    outl(PCI_CMD_PORT, PCI_CMD(0, 0, 0, PCI_CFG_DEVICE));
    d = inw(PCI_DATA_PORT);

    if (verbose) {
        puts("pci host bridge: ");
        putw(v);
        puts(":");
        putw(d);
        puts(" -> ");
    }

    if (v == 0x8086 && d == 0x1237) {
        if (verbose)
            puts("i440fx chipset (pc)\n");
        acpi_pm_init_pc();

    } else if (v == 0x8086 && d == 0x29c0) {
        if (verbose)
            puts("q35 chipset\n");
        acpi_pm_init_q35();

    } else {
        if (verbose)
            puts("unknown chipset\n");
    }
}
