use aarch64_cpu::registers::*;
use tock_registers::interfaces::Readable;

use crate::exception::{PrivilegeLevel, PrivilegeKind};

pub fn current_privilege_level() -> PrivilegeLevel {
    let el = CurrentEL.read_as_enum(CurrentEL::EL);
    match el {
        Some(CurrentEL::EL::Value::EL0) => PrivilegeLevel::new(
            PrivilegeKind::User,
            "EL0",
        ),
        Some(CurrentEL::EL::Value::EL1) => PrivilegeLevel::new(
            PrivilegeKind::Kernel,
            "EL1",
        ),
        Some(CurrentEL::EL::Value::EL2) => PrivilegeLevel::new(
            PrivilegeKind::Hypervisor,
            "EL2",
        ),
        _ => PrivilegeLevel::new(
            PrivilegeKind::Unknown,
            "Unknown",
        ),
    }
}