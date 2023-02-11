#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![no_main]
#![no_std]

mod panic;
mod arch;
mod board;
mod console;
mod sync;
mod error;
mod driver;

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
    use console::console;

    println!(
        "Emily version {}",
        env!("CARGO_PKG_VERSION")
    );
    println!("Running on: {}", board::BOARD_NAME);

    println!("Drivers:");
    driver::manager().dump();

    println!("Chars written: {}", console().chars_written());
    println!("Echoing input now");

    // Discard any spurious received characters before going into echo mode.
    console().clear_rx();
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}
