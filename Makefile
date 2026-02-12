# cargo audit
IGNORE	:= --ignore RUSTSEC-2023-0071	# rsa timing sidechannel (no secret keys here)

# variables
ARCH	:= $(shell uname -m)
PKGS	:= efi-tools igvm-tools varstore
BIOS32	:=
POST64	:=

ifeq ($(ARCH),x86_64)
PKGS	+= mefisto efi-apps
BIOS32	+= build-bios-32
BIOS32	+= build-none-libefi
BIOS32	+= build-none-varstore
POST64	+= cargo-post
endif

ifeq ($(ARCH),aarch64)
PKGS	+= efi-apps
endif

# target specific variables
doc-mefisto build-mefisto clippy-mefisto clippy-fix-mefisto release-mefisto package-mefisto : \
	TARGET := --target $(ARCH)-unknown-none --features efiloader
doc-efi-apps build-efi-apps clippy-efi-apps clippy-fix-efi-apps release-efi-apps package-efi-apps : \
	TARGET := --target $(ARCH)-unknown-uefi --features fdt
doc-efi-tools build-efi-tools clippy-efi-tools clippy-fix-efi-tools release-efi-tools package-efi-tools : \
	TARGET := --features udev
doc-varstore build-varstore clippy-varstore clippy-fix-varstore release-varstore package-varstore : \
	TARGET := --features std,json,pem

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

update:
	cargo update --verbose

audit:
	cargo audit $(IGNORE)

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
BIOS64	:= hello qemuinit igvmtest mefisto
BIOSELF	:= $(patsubst %,target/x86_64-unknown-none/debug/%,$(BIOS64))
BIOSBIN	:= $(patsubst %,target/x86_64-unknown-none/debug/%.bin,$(BIOS64))
IGVM	+= $(patsubst %,target/x86_64-unknown-none/debug/%.igvm,$(BIOS64))

cargo-post: $(BIOSBIN) $(IGVM)
$(BIOSELF): build-mefisto

%.bin: %
	objcopy -O binary $< $@

%.igvm: % build-igvm-tools
	target/debug/igvm-elfis --input $< --output $@

# non-cargo subdirs
build-bios-32: build-igvm-tools
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

# tmt + testing farm
GIT_BRANCH	:= $(shell git branch --show-current)
GIT_REMOTE	:= $(shell git config branch.$(GIT_BRANCH).remote)
GIT_URL		:= $(shell git config remote.$(GIT_REMOTE).url)

.PHONY: tmt
tmt:
	tmt run --all --verbose \
		report --how html

testing-farm-fedora:
	testing-farm request \
		--git-url $(GIT_URL) \
		--git-ref $(GIT_BRANCH) \
		--compose Fedora-43
