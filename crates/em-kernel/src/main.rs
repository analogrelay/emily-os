#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(const_option)]
#![feature(nonzero_min_max)]
#![feature(unchecked_math)]
#![no_main]
#![no_std]

use core::time::Duration;

use crate::exception::PrivilegeLevel;

mod panic;
mod arch;
mod board;
mod console;
mod sync;
mod error;
mod driver;
mod time;
mod exception;

/// Kernel Entry Point.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn kenter() -> ! {
    // Initialize the board, which will attach devices to the device manager
    board::init().expect("failed to initialize board");

    // Start drivers
    driver::manager().initialize();

    // Jump to safe code
    kmain()
}

fn kmain() -> ! {
    info!(
        "Emily version {}",
        env!("CARGO_PKG_VERSION")
    );
    info!("Board: {}", board::BOARD_NAME);

    let privl = PrivilegeLevel::current();
    info!("Current Privilege Level: {} - {}", privl.kind(), privl.name());

    info!("Timer resolution: {}ns", time::keeper().resolution().as_nanos());

    loop {
        info!("Sleeping 1 second");
        time::keeper().spin_for(Duration::from_secs(1));
    }
}
