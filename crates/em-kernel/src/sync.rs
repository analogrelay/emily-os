use crate::arch;

pub struct Mutex<T>(arch::sync::Mutex<T>) where T: ?Sized;

impl<T> Mutex<T> {
    /// Create an instance.
    pub const fn new(data: T) -> Self {
        Self(arch::sync::Mutex::new(data))
    }

    pub fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut T) -> R) -> R {
        self.0.lock(f)
    }
}

pub fn spin_while(cond: impl Fn() -> bool) {
    while cond() {
        arch::cpu::nop();
    }
}