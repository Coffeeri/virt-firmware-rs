#include "lib.h"


void outb(uint16_t port, uint8_t value)
{
    asm volatile ("outb %%al, %%dx" : : "d"(port), "a"(value));
}

void outw(uint16_t port, uint16_t value)
{
    asm volatile ("outw %%ax, %%dx" : : "d"(port), "a"(value));
}

void outl(uint16_t port, uint32_t value)
{
    asm volatile ("outl %%eax, %%dx" : : "d"(port), "a"(value));
}


uint8_t inb(uint16_t port)
{
    uint8_t value;
    asm volatile ("inb %%dx, %%al" : "=a"(value) : "d"(port));
    return value;
}

uint16_t inw(uint16_t port)
{
    uint16_t value;
    asm volatile ("inw %%dx, %%ax" : "=a"(value) : "d"(port));
    return value;
}

uint16_t inl(uint16_t port)
{
    uint32_t value;
    asm volatile ("inl %%dx, %%eax" : "=a"(value) : "d"(port));
    return value;
}

