TARGET            = aarch64-unknown-none-softfloat
KERNEL_BIN        = target/img/kernel-aarch64-raspi3.img
OBJDUMP_BINARY    = aarch64-none-elf-objdump
NM_BINARY         = aarch64-none-elf-nm
READELF_BINARY    = aarch64-none-elf-readelf
LD_SCRIPT_PATH    = $(shell pwd)/crates/em-kernel/src/board/raspi
RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53

export LD_SCRIPT_PATH