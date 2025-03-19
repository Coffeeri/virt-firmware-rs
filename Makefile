# variables
ARCH	:= $(shell uname -m)
PKGS	:= bios-64 efi-apps efi-tools

# target specific variables
doc-bios-64 build-bios-64 clippy-bios-64 release-bios-64 : TARGET := --target $(ARCH)-unknown-none
doc-efi-apps build-efi-apps clippy-efi-apps release-efi-apps : TARGET := --target $(ARCH)-unknown-uefi

# targets
all: build clippy test cargo-post
doc docs: cargo-doc
test tests: cargo-test
build: build-bios-32 cargo-build
clippy: cargo-clippy
release: build-bios-32 cargo-release

cargo-doc: $(patsubst %,doc-%,$(PKGS))
cargo-test: test-libefi
cargo-build: $(patsubst %,build-%,$(PKGS))
cargo-clippy: $(patsubst %,clippy-%,$(PKGS))
cargo-release: $(patsubst %,release-%,$(PKGS))

clean: clean-bios-32
	cargo clean
	find -name "*~" -exec rm -v "{}" ";"

realclean: clean
	rm -f Cargo.lock

# cargo post-processing
BIOS64	:= mini
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
push:
	git push
	sleep 10 # let gitlab create pipeline
	gitlab-ci

# cargo rules
doc-%:
	cargo doc --no-deps --package $* $(TARGET)

test-%:
	cargo test --package $* $(TARGET)

build-%:
	cargo build --package $* $(TARGET)

clippy-%:
	cargo clippy --package $* $(TARGET)

release-%:
	cargo build --release --package $* $(TARGET)
