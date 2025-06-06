# variables
CC	?= gcc
AS	:= $(CC)
LD	:= ld
OBJCOPY	:= objcopy

COMMON	:= -m32 -ffreestanding
ASFLAGS	:= $(COMMON)
CFLAGS	:= $(COMMON) -Os -g -Wall
CFLAGS	+= -mno-mmx -mno-sse -fpic
LDFLAGS	:= -melf_i386 --script=bios32.lds

TARGETS	:= hello.bin info.bin vmfwupdate.bin

# build targets
all: build

build: $(TARGETS)

hello.elf: hello.o entry32.o pci.o output.o ioport.o
info.elf: info.o entry32.o fwcfg.o pci.o output.o ioport.o
vmfwupdate.elf: vmfwupdate.o entry32.o fwcfg.o pci.o output.o ioport.o

clean:
	rm -f *.o *.elf *.bin

# test targets
run-hello: hello.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios hello.bin

run-info: info.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios info.bin

run-vmfwupdate: vmfwupdate.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios vmfwupdate.bin \
		-device vmfwupdate

# build rules
%.elf: %.o
	$(LD) $(LDFLAGS) -o $@ $^

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
