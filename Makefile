CONFIG_DIR = $(shell pwd)/.config

ARCH ?= $(shell cat $(CONFIG_DIR)/arch)
BOARD ?= $(shell cat $(CONFIG_DIR)/board)

include build/arch_$(ARCH).mk
include build/board_$(BOARD).mk

LAST_BUILD_CONFIG    = target/$(BOARD).build_config

KERNEL_MANIFEST = crates/em-kernel/Cargo.toml
KERNEL_ELF      = target/$(TARGET)/release/kernel
KERNEL_ELF_DEPS = $(filter-out %: ,$(file < $(KERNEL_ELF).d)) $(KERNEL_MANIFEST) $(LAST_BUILD_CONFIG)

RUSTFLAGS = $(RUSTC_MISC_ARGS)                   \
	-C link-arg=--library-path=$(LD_SCRIPT_PATH) \
	-C link-arg=--script=kernel.ld
FEATURES      = --features board_$(BOARD)
COMPILER_ARGS = --target=$(TARGET) \
	$(FEATURES)                    \
	--release

RUSTC   = cargo rustc $(COMPILER_ARGS)
CLIPPY  = cargo clippy $(COMPILER_ARGS)
OBJCOPY = rust-objcopy \
    --strip-all            \
    -O binary

.PHONY: all
all: kernel-bin

$(LAST_BUILD_CONFIG):
	@rm -f target/*.build_config
	@mkdir -p target
	@touch $(LAST_BUILD_CONFIG)

.PHONY: kernel-bin
kernel-bin: kernel-elf
	@mkdir -p target/img
	$(OBJCOPY) $(KERNEL_ELF) $(KERNEL_BIN)

.PHONY: kernel-elf
kernel-elf:
	RUSTFLAGS="$(RUSTFLAGS)" $(RUSTC) -p em-kernel
