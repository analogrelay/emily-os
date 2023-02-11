use crate::{driver, error::Error, console};

use super::{bcm, memory};

pub static PL011_UART: bcm::pl011_uart::PL011Uart = unsafe {
    bcm::pl011_uart::PL011Uart::new(memory::mmio::PL011_UART_START)
};
pub static GPIO: bcm::gpio::GPIO = unsafe {
    bcm::gpio::GPIO::new(memory::mmio::GPIO_START)
};

fn uart_post_init() -> Result<(), Error> {
    console::register_console(&PL011_UART);
    Ok(())
}

fn uart_init() -> Result<(), Error> {
    let descriptor = driver::DeviceDriverDescriptor::new(
        &PL011_UART,
        Some(uart_post_init)
    );
    driver::manager().install(descriptor);

    Ok(())
}

fn gpio_post_init() -> Result<(), Error> {
    GPIO.map_pl011_uart();
    Ok(())
}

fn gpio_init() -> Result<(), Error> {
    let descriptor = driver::DeviceDriverDescriptor::new(
        &GPIO,
        Some(gpio_post_init)
    );
    driver::manager().install(descriptor);

    Ok(())
}

pub fn init() -> Result<(), Error> {
    uart_init()?;
    gpio_init()?;

    Ok(())
}