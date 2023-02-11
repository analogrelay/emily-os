#[cfg(any(feature = "board_raspi3", feature = "board_raspi4"))]
mod raspi;

#[cfg(any(feature = "board_raspi3", feature = "board_raspi4"))]
pub use raspi::*;