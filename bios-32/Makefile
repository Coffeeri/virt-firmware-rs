# variables
USE_CLANG ?= no
ifeq ($(USE_CLANG),yes)
CC	:= clang --target=i386
LD	:= ld.lld
else ifneq ($(shell which x86_64-linux-gnu-gcc),)
CC	:= x86_64-linux-gnu-gcc -m32
LD	:= x86_64-linux-gnu-ld
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

TARGETS	:= hello-poweroff.bin hello-reset.bin hello-hltloop.bin info.bin
TARGETS	+= hello-poweroff.igvm hello-reset.igvm hello-hltloop.igvm info.igvm

# build targets
all: build

build: $(TARGETS)

hello-poweroff.elf: hello-poweroff.o entry32.o pci.o output.o ioport.o
hello-reset.elf: hello-reset.o entry32.o pci.o output.o ioport.o
hello-hltloop.elf: hello-hltloop.o entry32.o output.o ioport.o
info.elf: info.o entry32.o fwcfg.o pci.o output.o ioport.o

clean:
	rm -f *.o *.elf *.bin *.igvm

# test targets
run-hello: hello-poweroff.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios hello-poweroff.bin

run-info: info.bin
	qemu-system-x86_64 -enable-kvm -nographic -bios info.bin

# build rules
%.elf: %.o
	$(LD) $(LDFLAGS) -o $@ $^

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@

%.igvm: %.elf
	$(ELFIS) --input $< --output $@
