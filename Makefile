# cargo audit
IGNORE	:= --ignore RUSTSEC-2023-0071	# rsa timing sidechannel (no secret keys here)

# rust targets
SYSROOT	:= $(shell rustc --print sysroot)
RUSTLIB	:= $(SYSROOT)/lib/rustlib
TARGETS	:= $(patsubst $(RUSTLIB)/%,%,$(wildcard $(RUSTLIB)/*-*-*))

# cross compiler (used by cc crate)
ifneq ($(shell which x86_64-linux-gnu-gcc),)
export CC_x86_64_unknown_none := x86_64-linux-gnu-gcc
endif

# variables
ARCH		:= $(shell uname -m)
PKGS		:= efi-tools experimental igvm-tools varstore
TGT_BIOS32	:=
TGT_NONE	:=
TGT_POST	:=

ifeq ($(ARCH),x86_64)
TGT_BIOS32	+= build-bios-32
endif

ifneq ($(filter x86_64-unknown-none,$(TARGETS)),)
PKGS		+= mefisto
TGT_NONE	+= build-x-none-libefi
TGT_NONE	+= build-x-none-varstore
TGT_POST	+= cargo-post
endif

ifneq ($(filter $(ARCH)-unknown-uefi,$(TARGETS)),)
PKGS	+= efi-apps
endif

# cargo target names
TGT_TYPES	:= doc build clippy clippy-fix release install package
TGT_MEFISTO	:= $(patsubst %,%-mefisto,$(TGT_TYPES))
TGT_EFI_APPS	:= $(patsubst %,%-efi-apps,$(TGT_TYPES))
TGT_EFI_TOOLS	:= $(patsubst %,%-efi-tools,$(TGT_TYPES))
TGT_VARSTORE	:= $(patsubst %,%-varstore,$(TGT_TYPES))

# target specific variables
$(TGT_MEFISTO)    : TARGET := --target x86_64-unknown-none --features efiloader
$(TGT_EFI_APPS)   : TARGET := --target $(ARCH)-unknown-uefi --features fdt
$(TGT_EFI_TOOLS)  : TARGET := --features udev
$(TGT_VARSTORE)   : TARGET := --features std,json,pem

# targets
default: all
all: build clippy test docs
doc docs: cargo-doc
test tests: cargo-test
build: $(TGT_BIOS32) $(TGT_NONE) cargo-build $(TGT_POST)
clippy: cargo-clippy
clippy-fix: cargo-clippy-fix
release: $(TGT_BIOS32) cargo-release
install: cargo-install
package: cargo-package

update:
	cargo update --verbose

fedora: update
	rpms=$$(cargo update --verbose 2>&1 | awk '/available/ { printf("rust-%s-devel\n",$$2) }');\
		echo $$rpms;\
		dnf list --enablerepo=updates-testing $$rpms

duplicates:
	cargo tree --duplicates

audit:
	cargo audit $(IGNORE)

cargo-doc: $(patsubst %,doc-%,$(PKGS)) doc-libefi doc-libhw
cargo-test: test-libefi test-varstore test-igvm-tools
cargo-build: $(patsubst %,build-%,$(PKGS))
cargo-clippy: $(patsubst %,clippy-%,$(PKGS))
cargo-clippy-fix: $(patsubst %,clippy-fix-%,$(PKGS))
cargo-release: $(patsubst %,release-%,$(PKGS))
cargo-install: install-efi-tools install-igvm-tools
cargo-package: package-libhw package-libefi package-efi-tools package-igvm-tools package-varstore

clean: clean-bios-32
	cargo clean
	find -name "*~" -exec rm -v "{}" ";"

realclean: clean
	rm -f Cargo.lock

# cargo post-processing
BIOS64	:= hello qemuinit igvmtest mefisto snptest
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

build-x-none-%:
	cargo build --package virtfw-$* --target x86_64-unknown-none

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

ARCH		:= $(shell uname -m)
DISTRO_NAME	:= $(shell grep ^ID= /etc/os-release | tr -d '"' | cut -d= -f2)
DISTRO_VERSION	:= $(shell grep ^VERSION_ID= /etc/os-release | tr -d '"' | cut -d= -f2)
DISTRO		:= $(DISTRO_NAME)-$(DISTRO_VERSION)

HAVE_TERMINAL	:= $(shell test -t 0 && echo yes || echo no)
TMT_REPORT	:= --how html
ifeq ($(HAVE_TERMINAL),yes)
TMT_REPORT	+= --open
endif

.PHONY: tmt tmt-fips tmt-igvm tmt-snp
# default tests (in VM)
tmt:
	cd tmt; tmt --context distro=$(DISTRO) \
		run --all --verbose \
		report $(TMT_REPORT)

# fips mode tests (in centos stream -VM)
tmt-fips:
	cd tmt; tmt --context distro=centos-10 --context fips=yes \
		run --all --verbose \
		provision --how virtual --image centos-stream-10 \
		report $(TMT_REPORT)

# default tests (local)
tmt-local:
	cd tmt; tmt --feeling-safe --context distro=$(DISTRO) \
		run --all --verbose \
		provision --how local \
		report $(TMT_REPORT)

# snp boot tests (local, needs sev-snp host)
tmt-local-snp:
	cd tmt; tmt --feeling-safe --context distro=$(DISTRO) --context snp=yes \
		run --all --verbose \
		test --filter 'tag:snp' \
		provision --how local \
		report $(TMT_REPORT)

# fedora gating tests
tf-fedora:
	testing-farm request \
		--git-url $(GIT_URL) \
		--git-ref $(GIT_BRANCH) \
		--plan plans \
		--path tmt \
		--compose Fedora-44 \
		--context distro=fedora-44
