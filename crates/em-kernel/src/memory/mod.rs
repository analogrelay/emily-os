use core::{ops::RangeInclusive, fmt};

use crate::utils;

pub struct PhysicalAddress(pub usize);
pub struct VirtualAddress(pub usize);

#[derive(Debug)]
pub enum EnableError {
    AlreadyEnabled,
    UnsupportedGranule,
    Other(&'static str),
}

impl fmt::Display for EnableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnableError::AlreadyEnabled => write!(f, "MMU is already enabled"),
            EnableError::UnsupportedGranule => write!(f, "MMU does not support translation granule"),
            EnableError::Other(x) => write!(f, "{}", x),
        }
    }
}

pub trait MemoryManagementUnit {
    /// Called during kernel initialization to initialize the Memory Management Unit
    /// 
    /// # Safety
    /// Changes hardware global state and should only be called once, and at the appropriate time.
    unsafe fn enable(&self) -> Result<(), EnableError>;

    /// Indicates if the MMU is enabled
    fn is_enabled(&self) -> bool;
}

/// Describes the characteristics of a translation granule.
pub struct TranslationGranule<const GRANULE_SIZE: usize>;

impl<const GRANULE_SIZE: usize> TranslationGranule<GRANULE_SIZE> {
    /// The size of the translation granule, in bytes.
    pub const SIZE: usize = Self::size_checked();

    /// The granule's shift, given by log2(size)
    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(GRANULE_SIZE.is_power_of_two());
        GRANULE_SIZE
    }
}

/// Describes the characteristics of an address space.
pub struct AddressSpace<const AS_SIZE: usize>;

impl<const AS_SIZE: usize> AddressSpace<AS_SIZE> {
    /// The size of the address space, in bytes.
    pub const SIZE: usize = Self::size_checked();

    /// The address space's shift, given by log2(size)
    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(AS_SIZE.is_power_of_two());

        // Check architecture-specific restrictions
        Self::arch_check_address_space_size();

        AS_SIZE
    }
}

/// Translation types
#[derive(Clone)]
pub enum Translation {
    /// Virtual Addresses map directly to Physical Addresses
    Identity,

    /// Virtual Addresses are offset by the provided amount to get the Physical Address
    Offset(usize),
}

/// Identifies the type of a region of memory.
#[derive(Clone)]
pub enum MemoryType {
    /// The memory is standard RAM, eligible for storing arbitrary data and code.
    Normal,

    /// The memory is mapped to a device, and is used for communicating with the device.
    Device
}

/// Identifies the access permissions of a region of memory.
#[derive(Clone)]
pub enum MemoryAccess {
    /// The region can only be read.
    ReadOnly,

    /// The region can be read and written to.
    ReadWrite,
}

/// The attributes associated with a memory region.
#[derive(Clone)]
pub struct MemoryAttributes {
    /// The type of memory identified by this region.
    pub memory_type: MemoryType,

    /// The access permissions of this region.
    pub access: MemoryAccess,

    /// If set, memory in this region can be executed.
    pub executable: bool,
}

impl Default for MemoryAttributes {
    fn default() -> MemoryAttributes {
        MemoryAttributes {
            memory_type: MemoryType::Normal,
            access: MemoryAccess::ReadWrite,
            executable: false,
        }
    }
}

/// Describes the virtual address translation of a region of memory.
pub struct TranslationDescriptor {
    /// The name of the region
    pub name: &'static str,

    /// The virtual address range of the region
    pub virtual_range: fn() -> RangeInclusive<usize>,

    /// The translation method for the region, which determines the physical address range
    pub translation: Translation,

    /// The attributes of the region
    pub attributes: MemoryAttributes,
}

impl fmt::Display for TranslationDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Call the function to which self.range points, and dereference the result, which causes
        // Rust to copy the value.
        let start = *(self.virtual_range)().start();
        let end = *(self.virtual_range)().end();
        let size = end - start + 1;

        let (size, unit) = utils::size_human_readable_ceil(size);

        let attr = match self.attributes.memory_type {
            MemoryType::Normal => "N",
            MemoryType::Device => "D",
        };

        let acc_p = match self.attributes.access {
            MemoryAccess::ReadOnly => "RO",
            MemoryAccess::ReadWrite => "RW",
        };

        let xn = if self.attributes.executable {
            "PX"
        } else {
            "PXN"
        };

        write!(
            f,
            "      {:#010x} - {:#010x} | {: >3} {} | {: <3} {} {: <3} | {}",
            start, end, size, unit, attr, acc_p, xn, self.name
        )
    }
}

pub struct KernelVirtualMemoryLayout<const NUM_SPECIAL_RANGES: usize> {
    /// The last (inclusive) address of the address space.
    max_virtual_address: usize,

    /// Array of descriptors for non-standard (device) memory regions.
    inner: [TranslationDescriptor; NUM_SPECIAL_RANGES]
}


impl<const NUM_SPECIAL_RANGES: usize> KernelVirtualMemoryLayout<{ NUM_SPECIAL_RANGES }> {
    /// Create a new instance.
    pub const fn new(max: usize, layout: [TranslationDescriptor; NUM_SPECIAL_RANGES]) -> Self {
        Self {
            max_virtual_address: max,
            inner: layout,
        }
    }

    /// For a virtual address, find and return the physical output address and corresponding
    /// attributes.
    ///
    /// If the address is not found in `inner`, return an identity mapped default with normal
    /// cacheable DRAM attributes.
    pub fn virt_addr_properties(
        &self,
        virt_addr: usize,
    ) -> Result<(PhysicalAddress, MemoryAttributes), &'static str> {
        if virt_addr > self.max_virtual_address {
            return Err("Address out of range");
        }

        for i in self.inner.iter() {
            if (i.virtual_range)().contains(&virt_addr) {
                let output_addr = match i.translation {
                    Translation::Identity => virt_addr,
                    Translation::Offset(a) => a + (virt_addr - (i.virtual_range)().start()),
                };

                return Ok((PhysicalAddress(output_addr), i.attributes.clone()));
            }
        }

        Ok((PhysicalAddress(virt_addr), MemoryAttributes::default()))
    }

    /// Print the memory layout.
    pub fn print_layout(&self) {
        use crate::info;

        for i in self.inner.iter() {
            info!("{}", i);
        }
    }
}