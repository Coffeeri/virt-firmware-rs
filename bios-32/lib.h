#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// bios.lds
extern void *section_text;
extern void *section_data;
extern void *section_rodata;
extern void *section_end;
extern void *section_text16;
extern void *section_gdt;
extern void *section_gdtd;
extern void *section_entry16;

// ioport.c

void outb(uint16_t port, uint8_t value);
void outw(uint16_t port, uint16_t value);
void outl(uint16_t port, uint32_t value);
uint8_t inb(uint16_t port);
uint16_t inw(uint16_t port);
uint16_t inl(uint16_t port);

// output.c

void serial_init();
void serial_putc(char c);
void debugcon_putc(const char c);
void putc(char c);
void puts(const char *s);
void putb(uint8_t b);
void putw(uint16_t w);
void putl(uint32_t l);
void putq(uint64_t q);

// fwcfg.c

#define FWCFG_ITEM_SIGNATURE   0x00
#define FWCFG_ITEM_VERSION     0x01
#define FWCFG_ITEM_FILEDIR     0x19

void fwcfg_select(uint16_t item);
uint8_t fwcfg_read(void);
void fwcfg_read_bytes(uint8_t *buffer, uint32_t count);
uint16_t fwcfg_read_le16(void);
uint32_t fwcfg_read_le32(void);
uint64_t fwcfg_read_le64(void);
uint16_t fwcfg_read_be16(void);
uint32_t fwcfg_read_be32(void);
uint64_t fwcfg_read_be64(void);
bool fwcfg_detect(void);
void fwcfg_readdir(const char *search, uint16_t *ret_item, uint32_t *ret_size);
void fwcfg_hexdump(const char *file, uint32_t max);

// pci.c
void acpi_pm_init_pc(void);
void acpi_pm_init_q35(void);
void acpi_pm_poweroff(void);
void pci_init_info(bool verbose);
