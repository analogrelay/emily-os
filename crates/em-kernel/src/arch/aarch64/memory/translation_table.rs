use core::convert;

use tock_registers::{registers::InMemoryRegister, interfaces::{Readable, Writeable}, register_bitfields};

use crate::{board, memory::{PhysicalAddress, MemoryAttributes, MemoryType, MemoryAccess}};

use super::{Granule512MiB, Granule64KiB, mair};

// A table descriptor, as per ARMv8-A Architecture Reference Manual Figure D5-15.
register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        /// Physical address of the next descriptor.
        NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

        TYPE  OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

// A level 3 page descriptor, as per ARMv8-A Architecture Reference Manual Figure D5-17.
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        /// Unprivileged execute-never.
        UXN      OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next table descriptor (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

        /// Access flag.
        AF       OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Shareability field.
        SH       OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

        /// Access Permissions.
        AP       OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11
        ],

        /// Memory attributes index into the MAIR_EL1 register.
        AttrIndx OFFSET(2) NUMBITS(3) [],

        TYPE     OFFSET(1) NUMBITS(1) [
            Reserved_Invalid = 0,
            Page = 1
        ],

        VALID    OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

// A table descriptor for 64KiB granules.
#[repr(C)]
#[derive(Copy, Clone)]
struct TableDescriptor {
    value: u64,
}

impl TableDescriptor {
    /// Creates a new zeroed instance.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance pointing to the supplied address.
    pub fn from_next_level_table_address(next_level_table_address: PhysicalAddress) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);
        let shifted = next_level_table_address.0 >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
                + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
                + STAGE1_TABLE_DESCRIPTOR::VALID::True,
        );

        TableDescriptor { value: val.get() }
    }
}

/// A page descriptor for 64KiB granules.
#[repr(C)]
#[derive(Copy, Clone)]
struct PageDescriptor {
    value: u64,
}

impl PageDescriptor {
    /// Create a new zeroed instance.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance
    pub fn from_output_address(output_address: PhysicalAddress, attributes: &MemoryAttributes) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);
        let shifted = output_address.0 >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted as u64)
                + STAGE1_PAGE_DESCRIPTOR::AF::True
                + STAGE1_PAGE_DESCRIPTOR::TYPE::Page
                + STAGE1_PAGE_DESCRIPTOR::VALID::True
                + attributes.into(),
        );

        Self { value: val.get() }
    }
}

trait StartAddress {
    fn physical_start_address(&self) -> PhysicalAddress;
}

const NUM_LVL2_TABLES: usize = board::memory::KernelAddressSpace::SIZE >> Granule512MiB::SHIFT;

impl<T, const N:usize> StartAddress for [T; N] {
    fn physical_start_address(&self) -> PhysicalAddress {
        PhysicalAddress(self as *const T as usize)
    }
}

// Allows conversion from the generic MemoryAttrbiutes struct into a STAGE1_PAGE_DESCRIPTOR.
impl convert::From<&MemoryAttributes> for tock_registers::fields::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register> {
    fn from(attributes: &MemoryAttributes) -> Self {
        // Memory attributes.
        let mut desc = match attributes.memory_type {
            MemoryType::Normal => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mair::NORMAL)
            }
            MemoryType::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mair::DEVICE)
            }
        };

        // Access Permissions.
        desc += match attributes.access {
            MemoryAccess::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            MemoryAccess::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // The execute-never attribute is mapped to PXN in AArch64.
        desc += if attributes.executable {
            STAGE1_PAGE_DESCRIPTOR::PXN::False
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::True
        };

        // Always set unprivileged execute-never as long as userspace is not implemented yet.
        desc += STAGE1_PAGE_DESCRIPTOR::UXN::True;

        desc
    }
}

/// Represents all the translation tables for the kernel.
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    /// The level 3 tables (page descriptors).
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    /// The single level 2 table (table descriptors).
    lvl2: [TableDescriptor; NUM_TABLES],
}

pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
    /// Create an instance.
    pub const fn new() -> Self {
        // Can't have a zero-sized address space.
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
        }
    }

    /// Iterates over all static translation table entries and fills them at once.
    ///
    /// # Safety
    ///
    /// - Modifies a `static mut`. Ensure it only happens from here.
    pub unsafe fn populate_tt_entries(&mut self) -> Result<(), &'static str> {
        for (l2_nr, l2_entry) in self.lvl2.iter_mut().enumerate() {
            *l2_entry =
                TableDescriptor::from_next_level_table_address(self.lvl3[l2_nr].physical_start_address());

            for (l3_nr, l3_entry) in self.lvl3[l2_nr].iter_mut().enumerate() {
                let virt_addr = (l2_nr << Granule512MiB::SHIFT) + (l3_nr << Granule64KiB::SHIFT);

                let (phys_output_addr, attribute_fields) =
                    board::memory::virtual_memory_layout().virt_addr_properties(virt_addr)?;

                *l3_entry = PageDescriptor::from_output_address(phys_output_addr, &attribute_fields);
            }
        }

        Ok(())
    }

    /// The translation table's base address to be used for programming the MMU.
    pub fn base_address(&self) -> PhysicalAddress {
        self.lvl2.physical_start_address()
    }
}