use core::cell::UnsafeCell;

pub struct Mutex<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for Mutex<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for Mutex<T> where T: ?Sized + Send {}

impl<T> Mutex<T> {
    /// Create an instance.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut T) -> R) -> R {
        // TODO: Implement a real lock!
        let data = unsafe { &mut *self.data.get() };

        f(data)
    }
}