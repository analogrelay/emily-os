#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(const_option)]
#![feature(unchecked_math)]
#![feature(core_intrinsics)]
#![no_main]
#![no_std]

use core::time::Duration;

use memory::MemoryManagementUnit;

use crate::{exception::PrivilegeLevel, console::Write};

mod panic;
mod arch;
mod board;
mod console;
mod sync;
mod error;
mod driver;
mod time;
mod exception;
mod memory;
mod utils;

/// Kernel Entry Point.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn kenter() -> ! {
    info!("Initializing MMU");

    if let Err(e) = arch::memory::mmu().enable() {
        panic!("Failed to enable MMU: {}", e);
    }

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

    info!("MMU online. Special regions:");
    board::memory::virtual_memory_layout().print_layout();

    let privl = PrivilegeLevel::current();
    info!("Current Privilege Level: {} - {}", privl.kind(), privl.name());

    info!("Timer resolution: {}ns", time::keeper().resolution().as_nanos());

    let remapped_uart = unsafe { board::bcm::pl011_uart::PL011Uart::new(0x1FFF_1000) };
    writeln!(
        remapped_uart,
        "[     !!!    ] Writing through the remapped UART at 0x1FFF_1000"
    )
    .unwrap();

    loop {
        info!("Sleeping 1 second");
        time::keeper().spin_for(Duration::from_secs(1));
    }
}
