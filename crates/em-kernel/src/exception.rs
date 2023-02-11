use crate::arch;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
/// An architecture-independent description of the current privilege level type
pub enum PrivilegeKind {
    /// The lowest level of privilege, where user-level applications reside
    User,

    /// A Kernel, the second-highest level of privilege, and where operating system kernels usually reside.
    Kernel,

    /// A Hypervisor, often the highest level of privilege in the processor.
    /// Usually, only Hypervisors reside at this level.
    Hypervisor,

    /// An unknown privilege level
    Unknown,
}

impl core::fmt::Display for PrivilegeKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Kernel => write!(f, "Kernel"),
            Self::Hypervisor => write!(f, "Hypervisor"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Represents the current privilege level of the processor.
pub struct PrivilegeLevel {
    kind: PrivilegeKind,
    name: &'static str,
}

impl PrivilegeLevel {
    pub fn new(typ: PrivilegeKind, name: &'static str) -> Self {
        Self { kind: typ, name }
    }

    pub fn current() -> PrivilegeLevel {
        arch::exception::current_privilege_level()
    }

    /// Gets an architecture-dependent name for the current privilege level
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the architecture-independent type of the current privilege level
    pub fn kind(&self) -> PrivilegeKind {
        self.kind
    }
}