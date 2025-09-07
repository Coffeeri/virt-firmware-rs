# variables
ARCH	:= $(shell uname -m)
PKGS	:= efi-tools igvm-tools varstore
BIOS32	:=
POST64	:=

ifeq ($(ARCH),x86_64)
PKGS	+= bios-64 efi-apps
BIOS32	+= build-bios-32
BIOS32	+= build-none-libefi
BIOS32	+= build-none-varstore
POST64	+= cargo-post
endif

ifeq ($(ARCH),aarch64)
PKGS	+= efi-apps
endif

# target specific variables
doc-bios-64 build-bios-64 clippy-bios-64 clippy-fix-bios-64 release-bios-64 package-bios-64 : TARGET := --target $(ARCH)-unknown-none
doc-efi-apps build-efi-apps clippy-efi-apps clippy-fix-efi-apps release-efi-apps package-efi-apps : TARGET := --target $(ARCH)-unknown-uefi --features fdt
doc-efi-tools build-efi-tools clippy-efi-tools clippy-fix-efi-tools release-efi-tools package-efi-tools : TARGET := --features udev
doc-varstore build-varstore clippy-varstore clippy-fix-varstore release-varstore package-varstore : TARGET := --features std

# targets
default: all
all: build clippy test docs
doc docs: cargo-doc
test tests: cargo-test
build: $(BIOS32) cargo-build $(POST64)
clippy: cargo-clippy
clippy-fix: cargo-clippy-fix
release: $(BIOS32) cargo-release
install: cargo-install
package: cargo-package

cargo-doc: $(patsubst %,doc-%,$(PKGS)) doc-libefi doc-libhw
cargo-test: test-libefi test-varstore test-igvm-tools
cargo-build: $(patsubst %,build-%,$(PKGS))
cargo-clippy: $(patsubst %,clippy-%,$(PKGS))
cargo-clippy-fix: $(patsubst %,clippy-fix-%,$(PKGS))
cargo-release: $(patsubst %,release-%,$(PKGS))
cargo-install: install-efi-tools install-igvm-tools
cargo-package: package-libefi package-efi-tools package-igvm-tools

clean: clean-bios-32
	cargo clean
	find -name "*~" -exec rm -v "{}" ";"

realclean: clean
	rm -f Cargo.lock

# cargo post-processing
BIOS64	:= mini hello
BIOSELF	:= $(patsubst %,target/x86_64-unknown-none/debug/%,$(BIOS64))
BIOSBIN	:= $(patsubst %,target/x86_64-unknown-none/debug/%.bin,$(BIOS64))
IGVM	:= $(patsubst %,target/x86_64-unknown-none/debug/%.igvm,$(BIOS64))

cargo-post: $(BIOSBIN) $(IGVM)
$(BIOSELF): build-bios-64

%.bin: %
	objcopy -O binary $< $@

%.igvm: %.bin build-igvm-tools
	target/debug/igvm-wrap --input $< --output $@

# non-cargo subdirs
build-bios-32:
	make -C bios-32 build

clean-bios-32:
	make -C bios-32 clean

# maintainer target
push: all
	git push
	sleep 10 # let gitlab create pipeline
	gitlab-ci

# cargo rules
doc-%:
	cargo doc --no-deps --package virtfw-$* $(TARGET)

test-%:
	cargo test --package virtfw-$* $(TARGET)
	cargo test --package virtfw-$* --all-features $(TARGET)

build-%:
	cargo build --package virtfw-$* $(TARGET)

build-none-%:
	cargo build --package virtfw-$* --target $(ARCH)-unknown-none

clippy-%:
	cargo clippy --package virtfw-$* $(TARGET)

clippy-fix-%:
	cargo clippy --fix --package virtfw-$* $(TARGET)

release-%:
	cargo build --release --package virtfw-$* $(TARGET)

install-%:
	cargo install --path $* $(TARGET)

package-%:
	cargo package --package virtfw-$* $(TARGET)

# include makefiles with run-* convinience targets
include scripts/Make.qemu.bare
include scripts/Make.qemu.uefi
