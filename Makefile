# variables
ARCH	:= $(shell uname -m)
UEFI	:= $(ARCH)-unknown-uefi
BARE	:= $(ARCH)-unknown-none
PKGS	:= bios-64 efi-apps

# target specific variables
build-bios-64 clippy-bios-64 : TARGET := $(BARE)
build-efi-apps clippy-efi-apps : TARGET := $(UEFI)

# targets
default: build clippy
build: $(patsubst %,build-%,$(PKGS))
clippy: $(patsubst %,clippy-%,$(PKGS))

clean:
	cargo clean
	find -name "*~" -exec rm -v "{}" ";"

realclean: clean
	rm -f Cargo.lock

# rules
build-%:
	cargo build --package $* --target $(TARGET)

clippy-%:
	cargo clippy --package $* --target $(TARGET)

