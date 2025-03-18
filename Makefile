# variables
ARCH	:= $(shell uname -m)
UEFI	:= $(ARCH)-unknown-uefi
BARE	:= $(ARCH)-unknown-none
PKGS	:= bios-64 efi-apps

# target specific variables
build-bios-64 clippy-bios-64 : TARGET := $(BARE)
build-efi-apps clippy-efi-apps : TARGET := $(UEFI)

# targets
all build: build-bios-32 cargo-build cargo-clippy
cargo-build: $(patsubst %,build-%,$(PKGS))
cargo-clippy: $(patsubst %,clippy-%,$(PKGS))

clean: clean-bios-32
	cargo clean
	find -name "*~" -exec rm -v "{}" ";"

realclean: clean
	rm -f Cargo.lock

# non-cargo subdirs
build-bios-32:
	make -C bios-32 build

clean-bios-32:
	make -C bios-32 clean

# rules
build-%:
	cargo build --package $* --target $(TARGET)

clippy-%:
	cargo clippy --package $* --target $(TARGET)
