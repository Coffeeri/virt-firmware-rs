#include "lib.h"

#define SERIAL_BASE    0x3f8
#define DEBUGCON_PORT  0x402

void serial_init()
{
    outb(SERIAL_BASE + 3, 0x03);  // 8N1
    outb(SERIAL_BASE + 1, 0);     // no irq
}

void serial_putc(char c)
{
    uint8_t lsr;

    do {
        lsr = inb(SERIAL_BASE + 5);
    } while (!(lsr & 0x20));

    outb(SERIAL_BASE + 0, c);
}

void debugcon_putc(char c)
{
    outb(DEBUGCON_PORT, c);
}

void putc(char c)
{
    debugcon_putc(c);
    serial_putc(c);
}

void puts(const char *s)
{
    while (*s) {
        putc(*s++);
    }
}

void putsln(const char *s)
{
    puts(s);
    puts("\r\n");
}

void putb(uint8_t b)
{
    static const char map[] = {
        '0', '1', '2', '3', '4', '5', '6', '7',
        '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    };

    putc(map[(b >> 4) & 0x0f]);
    putc(map[(b >> 0) & 0x0f]);
}

void putw(uint16_t w)
{
    putb((w >> 8) & 0xff);
    putb((w >> 0) & 0xff);
}

void putl(uint32_t l)
{
    putw((l >> 16) & 0xffff);
    putw((l >> 0) & 0xffff);
}

void putq(uint64_t q)
{
    putl((q >> 32) & 0xffffffff);
    putc(' ');
    putl((q >> 0) & 0xffffffff);
}
