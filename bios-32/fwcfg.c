#include "lib.h"

#define IOPORT_ITEM     0x510
#define IOPORT_DATA     0x511
#define IOPORT_DMA      0x514

void fwcfg_select(uint16_t item)
{
    outw(IOPORT_ITEM, item);
}

uint8_t fwcfg_read(void)
{
    return inb(IOPORT_DATA);
}

void fwcfg_read_bytes(uint8_t *buffer, uint32_t count)
{
    for (uint32_t i = 0; i < count; i++) {
        buffer[i] = fwcfg_read();
    }
}

uint16_t fwcfg_read_le16(void)
{
    uint16_t b1 = fwcfg_read();
    uint16_t b2 = fwcfg_read();
    return b1 | (b2 << 8);
}

uint32_t fwcfg_read_le32(void)
{
    uint32_t w1 = fwcfg_read_le16();
    uint32_t w2 = fwcfg_read_le16();
    return w1 | (w2 << 16);
}

uint64_t fwcfg_read_le64(void)
{
    uint64_t l1 = fwcfg_read_le32();
    uint64_t l2 = fwcfg_read_le32();
    return l1 | (l2 << 32);
}

uint16_t fwcfg_read_be16(void)
{
    uint16_t b1 = fwcfg_read();
    uint16_t b2 = fwcfg_read();
    return b2 | (b1 << 8);
}

uint32_t fwcfg_read_be32(void)
{
    uint32_t w1 = fwcfg_read_be16();
    uint32_t w2 = fwcfg_read_be16();
    return w2 | (w1 << 16);
}

uint64_t fwcfg_read_be64(void)
{
    uint64_t l1 = fwcfg_read_be32();
    uint64_t l2 = fwcfg_read_be32();
    return l2 | (l1 << 32);
}

bool fwcfg_detect(void)
{
    fwcfg_select(FWCFG_ITEM_SIGNATURE);
    uint32_t sig = fwcfg_read_le32();
    if (sig != 0x554d4551) {
        return false;
    }

    fwcfg_select(FWCFG_ITEM_VERSION);
    uint32_t ver = fwcfg_read_le32();

    puts("fwcfg: signature=");
    putl(sig);
    puts(" version=");
    putl(ver);
    putsln("");

    return true;
}

static bool streq(const char *a, const char *b)
{
    while (*a && *b) {
        if (*a != *b)
            return false;
        a++; b++;
    }
    return true;
}

void fwcfg_readdir(const char *search, uint16_t *ret_item, uint32_t *ret_size)
{
    uint32_t count;
    uint32_t size;
    uint16_t item;
    uint8_t name[56];
    bool found;

    fwcfg_select(FWCFG_ITEM_FILEDIR);
    count = fwcfg_read_be32();

    if (!search) {
        puts("fwcfg: file count=");
        putl(count);
        putsln("");
    }

    for (uint32_t i = 0; i < count; i++) {
        size = fwcfg_read_be32();
        item = fwcfg_read_be16();
        fwcfg_read_be16();
        fwcfg_read_bytes(name, sizeof(name));
        found = search ? streq((char*)name, search) : false;

        if (!search || found) {
            puts("fwcfg: file size=");
            putl(size);
            puts(" item=");
            putw(item);
            puts(" name='");
            puts((char*)name);
            putsln("'");
        }

        if (found) {
            *ret_item = item;
            *ret_size = size;
            return;
        }
    }

    if (search) {
        puts("fwcfg: file not found ('");
        puts(search);
        putsln("')");
    }
}

void fwcfg_hexdump(const char *file, uint32_t max)
{
    uint16_t item = 0;
    uint32_t size;
    uint32_t i;
    uint8_t byte;

    fwcfg_readdir(file, &item, &size);
    if (!item)
        return;

    fwcfg_select(item);
    for (i = 0; i < size && i < max; i++) {
        if ((i % 16) == 0) {
            puts(file);
            putc(' ');
            putw(i);
            putc(':');
        }
        if ((i % 4) == 0) {
            putc(' ');
        }
        byte = fwcfg_read();
        putc(' ');
        putb(byte);
        if ((i % 16) == 15) {
            putsln("");
        }
    }
    if ((i % 16) != 0) {
        putsln("");
    }
}
