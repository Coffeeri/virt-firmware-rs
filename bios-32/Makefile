CC	?= gcc
LD	:= ld
OBJCOPY	:= objcopy

ASFLAGS	:= -m32
CFLAGS	:= -m32 -ffreestanding -Os -g -Wall
CFLAGS	+= -mno-mmx -mno-sse -fpic
LDFLAGS	:= -melf_i386 --script=bios32.lds

TARGETS	:= hello.bin info.bin vmfwupdate.bin

all: build

build: $(TARGETS)

hello.elf: hello.o entry32.o pci.o output.o ioport.o
info.elf: info.o entry32.o fwcfg.o pci.o output.o ioport.o
vmfwupdate.elf: vmfwupdate.o entry32.o fwcfg.o pci.o output.o ioport.o

clean:
	rm -f *.o *.elf *.bin

%.elf: %.o
	$(LD) $(LDFLAGS) -o $@ $^

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@

run-hello: hello.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios hello.bin

run-info: info.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios info.bin

run-vmfwupdate: vmfwupdate.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios vmfwupdate.bin \
		-device vmfwupdate
