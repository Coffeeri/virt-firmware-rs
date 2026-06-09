# variables
ifneq ($(shell which x86_64-linux-gnu-gcc),)
CC	:= x86_64-linux-gnu-gcc -m32
LD	:= x86_64-linux-gnu-ld
else ifneq ($(shell which clang),)
CC	:= clang --target=i386
LD	:= ld.lld
else
CC	:= gcc -m32
LD	:= ld
endif

AS	:= $(CC)
OBJCOPY	:= objcopy
ELFIS	:= ../target/debug/igvm-elfis

COMMON	:= -ffreestanding -nostdlib
ASFLAGS	:= $(COMMON)
CFLAGS	:= $(COMMON) -Os -g -Wall
CFLAGS	+= -mno-mmx -mno-sse -fpic
LDFLAGS	:= -melf_i386 --script=bios32.lds

TARGETS	:= hello-poweroff.bin info.bin vmfwupdate.bin
TARGETS	+= hello-poweroff.igvm info.igvm vmfwupdate.igvm

# build targets
all: build

build: $(TARGETS)

hello-poweroff.elf: hello-poweroff.o entry32.o pci.o output.o ioport.o
info.elf: info.o entry32.o fwcfg.o pci.o output.o ioport.o
vmfwupdate.elf: vmfwupdate.o entry32.o fwcfg.o pci.o output.o ioport.o

clean:
	rm -f *.o *.elf *.bin *.igvm

# test targets
run-hello: hello-poweroff.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios hello-poweroff.bin

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

%.igvm: %.elf
	$(ELFIS) --input $< --output $@
