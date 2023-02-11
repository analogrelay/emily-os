use core;

use crate::sync::Mutex;

pub trait Write {
    /// Write a single character.
    fn write_char(&self, c: char);

    /// Write a Rust format string.
    fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result;

    /// Block until the last buffered character has been physically put on the TX wire.
    fn flush(&self);
}

pub trait Read {
    /// Read a single character.
    fn read_char(&self) -> char {
        ' '
    }

    /// Clear RX buffers, if any.
    fn clear_rx(&self);
}

pub trait Console: Read + Write {
    fn chars_written(&self) -> usize {
        0
    }

    fn chars_read(&self) -> usize {
        0
    }
}

struct NullConsole;
static NULL_CONSOLE: NullConsole = NullConsole;

impl Write for NullConsole {
    fn write_char(&self, _c: char) {}

    fn write_fmt(&self, _args: core::fmt::Arguments) -> core::fmt::Result {
        core::fmt::Result::Ok(())
    }

    fn flush(&self) {}
}

impl Read for NullConsole {
    fn clear_rx(&self) {}
}
impl Console for NullConsole {}

static CURRENT_CONSOLE: Mutex<&'static (dyn Console + Sync)> = Mutex::new(&NULL_CONSOLE);

pub fn register_console(new_console: &'static (dyn Console + Sync)) {
    CURRENT_CONSOLE.lock(|con| *con = new_console);
}

pub fn console() -> &'static (dyn Console + Sync) {
    CURRENT_CONSOLE.lock(|con| *con)
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    console().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::console::_print(format_args_nl!($($arg)*));
    })
}

/// Prints an info, with a newline.
#[macro_export]
macro_rules! info {
    ($string:expr) => ({
        let timestamp = $crate::time::keeper().uptime();

        $crate::console::_print(format_args_nl!(
            concat!("[  {:>3}.{:06}] ", $string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::time::keeper().uptime();

        $crate::console::_print(format_args_nl!(
            concat!("[  {:>3}.{:06}] ", $format_string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*
        ));
    })
}

/// Prints a warning, with a newline.
#[macro_export]
macro_rules! warn {
    ($string:expr) => ({
        let timestamp = $crate::time::keeper().uptime();

        $crate::console::_print(format_args_nl!(
            concat!("[W {:>3}.{:06}] ", $string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::time::keeper().uptime();

        $crate::console::_print(format_args_nl!(
            concat!("[W {:>3}.{:06}] ", $format_string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*
        ));
    })
}