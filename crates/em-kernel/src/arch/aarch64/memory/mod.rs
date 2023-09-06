use core::intrinsics::unlikely;

use aarch64_cpu::{registers::{TCR_EL1, MAIR_EL1, SCTLR_EL1, ID_AA64MMFR0_EL1, TTBR0_EL1}, asm::barrier};
use tock_registers::interfaces::{Readable, ReadWriteable, Writeable};

use crate::{memory::{TranslationGranule, AddressSpace, MemoryManagementUnit, EnableError}, board};

use self::translation_table::KernelTranslationTable;

mod translation_table;

struct AArch64MemoryManagementUnit;

pub type Granule512MiB = TranslationGranule<{ 512 * 1024 * 1024 }>;
pub type Granule64KiB = TranslationGranule<{ 64 * 1024 }>;

mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

static mut KERNEL_TABLES: KernelTranslationTable = KernelTranslationTable::new();
static MMU: AArch64MemoryManagementUnit = AArch64MemoryManagementUnit;

impl<const AS_SIZE: usize> AddressSpace<AS_SIZE> {
    /// Checks for architectural restrictions.
    pub const fn arch_check_address_space_size() {
        // Size must be at least one full 512 MiB table.
        assert!((AS_SIZE % Granule512MiB::SIZE) == 0);

        // Check for 48 bit virtual address size as maximum, which is supported by any ARMv8
        // version.
        assert!(AS_SIZE <= (1 << 48));
    }
}

impl AArch64MemoryManagementUnit {
    /// Sets up the MAIR_EL1 register.
    fn set_up_mair(&self) {
        // Define the memory types being mapped.
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM.
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
            MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +

            // Attribute 0 - Device.
            MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }

    /// Configure various settings of stage 1 of the EL1 translation regime.
    fn configure_translation_control(&self) {
        let t0sz = (64 - board::memory::KernelAddressSpace::SHIFT) as u64;

        TCR_EL1.write(
            TCR_EL1::TBI0::Used
                + TCR_EL1::IPS::Bits_40
                + TCR_EL1::TG0::KiB_64
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD0::EnableTTBR0Walks
                + TCR_EL1::A1::TTBR0
                + TCR_EL1::T0SZ.val(t0sz)
                + TCR_EL1::EPD1::DisableTTBR1Walks,
        );
    }
}

pub fn mmu() -> &'static impl MemoryManagementUnit {
    &MMU
}

impl MemoryManagementUnit for AArch64MemoryManagementUnit {
    unsafe fn enable(&self) -> Result<(), EnableError> {
        if unlikely(self.is_enabled()) {
            return Err(EnableError::AlreadyEnabled)
        }

        // Fail early if the translation granule is not supported
        if unlikely(!ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported)) {
            return Err(EnableError::UnsupportedGranule);
        }

        // Configure memory attributes
        self.set_up_mair();

        // Load translation tables
        KERNEL_TABLES
            .populate_tt_entries()
            .map_err(EnableError::Other)?;

        // Set the Translation Table Base Register (TTBR)
        TTBR0_EL1.set_baddr(KERNEL_TABLES.base_address().0 as u64);

        self.configure_translation_control();

        // Enable the MMU!
        // First, establish an Instruction barrier, to force all previous changes to be seen before processing further instructions
        barrier::isb(barrier::SY);

        // Enable the MMU by setting the bit in the System Control Register.
        // We're also marking data and instructions as cachable.
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

        // Another instruction barrier, so that the MMU is active before further instructions are executed
        barrier::isb(barrier::SY);

        Ok(())
    }

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable)
    }
}