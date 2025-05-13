# variables
ARCH	:= $(shell uname -m)
PKGS	:= efi-apps efi-tools igvm-tools varstore
BIOS32	:=
POST64	:=

ifeq ($(ARCH),x86_64)
PKGS	+= bios-64
BIOS32	+= build-bios-32 build-none
POST64	+= cargo-post
endif

# target specific variables
doc-bios-64 build-bios-64 clippy-bios-64 release-bios-64 : TARGET := --target $(ARCH)-unknown-none
doc-efi-apps build-efi-apps clippy-efi-apps release-efi-apps : TARGET := --target $(ARCH)-unknown-uefi

# targets
default: build clippy $(POST64)
all: build clippy test package $(POST64)
doc docs: cargo-doc
test tests: cargo-test
build: $(BIOS32) cargo-build
clippy: cargo-clippy
release: $(BIOS32) cargo-release
install: cargo-install
package: cargo-package

cargo-doc: $(patsubst %,doc-%,$(PKGS))
cargo-test: test-libefi test-varstore
cargo-build: $(patsubst %,build-%,$(PKGS))
cargo-clippy: $(patsubst %,clippy-%,$(PKGS))
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
BIOSBIN	:= $(patsubst %,target/x86_64-unknown-none/debug/%.bin,$(BIOS64))
BIOSELF	:= $(patsubst %,target/x86_64-unknown-none/debug/%,$(BIOS64))

cargo-post: $(BIOSBIN)
$(BIOSELF): build-bios-64

%.bin: %
	objcopy -O binary $< $@

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

build-none:
	cargo build --package virtfw-libefi --no-default-features --target $(ARCH)-unknown-none
	cargo build --package virtfw-varstore --no-default-features --target $(ARCH)-unknown-none

clippy-%:
	cargo clippy --package virtfw-$* $(TARGET)

release-%:
	cargo build --release --package virtfw-$* $(TARGET)

install-%:
	cargo install --path $* $(TARGET)

package-%:
	cargo package --package virtfw-$* $(TARGET)

# include makefiles with run-* convinience targets
include scripts/Make.qemu.bare
include scripts/Make.qemu.uefi
