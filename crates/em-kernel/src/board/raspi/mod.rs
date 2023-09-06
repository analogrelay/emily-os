use core::sync::atomic::{AtomicBool, Ordering};

use crate::error::Error;

pub mod cpu;

pub mod devices;
pub mod memory;
pub mod bcm;

#[cfg(feature = "board_raspi3")]
pub const BOARD_NAME: &str = "Raspberry Pi 3";

#[cfg(feature = "board_raspi4")]
pub const BOARD_NAME: &str = "Raspberry Pi 4";

pub unsafe fn init() -> Result<(), Error> {
    static INIT_COMPLETE: AtomicBool = AtomicBool::new(false);
    if INIT_COMPLETE.load(Ordering::Relaxed) {
        return Err("Board already initialized".into());
    }

    devices::init()?;

    INIT_COMPLETE.store(true, Ordering::Relaxed);
    Ok(())
}