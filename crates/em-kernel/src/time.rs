use core::time::Duration;

use crate::arch;

pub struct TimeKeeper;

impl TimeKeeper {
    /// Create an instance.
    pub const fn new() -> Self {
        Self
    }

    /// The timer's resolution.
    pub fn resolution(&self) -> Duration {
        arch::time::resolution()
    }

    /// The uptime since power-on of the device.
    ///
    /// This includes time consumed by firmware and bootloaders.
    pub fn uptime(&self) -> Duration {
        arch::time::uptime()
    }

    /// Spin for a given duration.
    pub fn spin_for(&self, duration: Duration) {
        arch::time::spin_for(duration)
    }
}

static TIME_MANAGER: TimeKeeper = TimeKeeper::new();

pub fn keeper() -> &'static TimeKeeper {
    &TIME_MANAGER
}