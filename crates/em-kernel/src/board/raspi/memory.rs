use core::cell::UnsafeCell;

use crate::memory::{KernelVirtualMemoryLayout, TranslationDescriptor, MemoryAttributes, MemoryType, MemoryAccess, Translation, AddressSpace};

// Symbols from the linker script.
extern "Rust" {
    static __code_start: UnsafeCell<()>;
    static __code_end_exclusive: UnsafeCell<()>;
}

// Defines memory layout
pub const GPIO_OFFSET:         usize = 0x0020_0000;
pub const UART_OFFSET:         usize = 0x0020_1000;
const END_INCLUSIVE:       usize = 0xFFFF_FFFF;

/// Physical devices.
#[cfg(feature = "board_raspi3")]
pub mod mmio {
    use super::*;

    pub const START:            usize =         0x3F00_0000;
    pub const GPIO_START:       usize = START + GPIO_OFFSET;
    pub const PL011_UART_START: usize = START + UART_OFFSET;
}

/// Physical devices.
#[cfg(feature = "board_raspi4")]
pub mod mmio {
    use super::*;

    pub const START:            usize =         0xFE00_0000;
    pub const GPIO_START:       usize = START + GPIO_OFFSET;
    pub const PL011_UART_START: usize = START + UART_OFFSET;
}

/// Start page address of the code segment.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn code_start() -> usize {
    unsafe { __code_start.get() as usize }
}

/// Exclusive end page address of the code segment.
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn code_end_exclusive() -> usize {
    unsafe { __code_end_exclusive.get() as usize }
}

/// The virtual memory layout.
///
/// The layout must contain only special ranges, aka anything that is _not_ normal cacheable DRAM.
/// It is agnostic of the paging granularity that the architecture's MMU will use.
pub static LAYOUT: KernelVirtualMemoryLayout<3> = KernelVirtualMemoryLayout::new(
    END_INCLUSIVE,
    [
        TranslationDescriptor {
            name: "Kernel code and RO data",
            virtual_range: || code_start()..=code_end_exclusive(),
            translation: Translation::Identity,
            attributes: MemoryAttributes {
                memory_type: MemoryType::Normal,
                access: MemoryAccess::ReadOnly,
                executable: false,
            },
        },
        TranslationDescriptor {
            name: "Remapped Device MMIO",
            virtual_range: || 0x1FFF_0000..=0x1FFF_FFFF,
            translation: Translation::Offset(mmio::START + 0x20_0000),
            attributes: MemoryAttributes {
                memory_type: MemoryType::Device,
                access: MemoryAccess::ReadWrite,
                executable: false,
            },
        },
        TranslationDescriptor {
            name: "Device MMIO",
            virtual_range: || mmio::START..=END_INCLUSIVE,
            translation: Translation::Identity,
            attributes: MemoryAttributes {
                memory_type: MemoryType::Device,
                access: MemoryAccess::ReadWrite,
                executable: false,
            },
        },
    ],
);

/// The physical address space available to the kernel on this board.
pub type KernelAddressSpace = AddressSpace<{ END_INCLUSIVE + 1 }>;

/// Gets the virtual memory layout used on this board.
pub fn virtual_memory_layout() -> &'static KernelVirtualMemoryLayout<3> {
    &LAYOUT
}