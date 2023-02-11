use aarch64_cpu;

pub use aarch64_cpu::asm::nop;

#[inline(always)]
pub fn halt() -> ! {
    loop {
        aarch64_cpu::asm::wfe();
    }
}