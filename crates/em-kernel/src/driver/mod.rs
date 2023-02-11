use crate::{error::Error, sync::Mutex, println, info};

// TODO: Bump allocator and dynamic allocation!
const MAX_DRIVERS: usize = 5;

struct DriverManagerInner {
    next_index: usize,
    descriptors: [Option<DeviceDriverDescriptor>; MAX_DRIVERS],
}

impl DriverManagerInner {
    pub const fn new() -> Self {
        Self {
            next_index: 0,
            descriptors: [None; MAX_DRIVERS],
        }
    }
}

pub trait DeviceDriver {
    fn name(&self) -> &'static str;

    unsafe fn init(&self) -> Result<(), Error> {
        // Some drivers don't need to be initialized!
        Ok(())
    }
}

pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), Error>;

#[derive(Copy, Clone)]
pub struct DeviceDriverDescriptor {
    driver: &'static (dyn DeviceDriver + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
}

impl DeviceDriverDescriptor {
    pub const fn new(
        driver: &'static (dyn DeviceDriver + Sync),
        post_init_callback: Option<DeviceDriverPostInitCallback>,
    ) -> Self {
        Self {
            driver,
            post_init_callback,
        }
    }
}

pub struct DriverManager {
    inner: Mutex<DriverManagerInner>,
}

impl DriverManager {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(DriverManagerInner::new()),
        }
    }

    pub fn install(&self, descriptor: DeviceDriverDescriptor) {
        self.inner.lock(|inner| {
            if inner.next_index >= MAX_DRIVERS {
                panic!("Too many drivers installed!");
            }
            inner.descriptors[inner.next_index] = Some(descriptor);
            inner.next_index += 1;
        })
    }

    pub unsafe fn initialize(&self) {
        self.for_each_descriptor(|descriptor| {
            if let Err(x) = descriptor.driver.init() {
                panic!(
                    "Failed to initialize driver '{}': {}", 
                    descriptor.driver.name(),
                    x);
            }

            if let Some(callback) = descriptor.post_init_callback {
                if let Err(x) = callback() {
                    panic!(
                        "Failed to post-initialize driver '{}': {}",
                        descriptor.driver.name(),
                        x);
                }
            }
        });

        // The drivers are part of how we print to the console, so we had to initialize them first.
        // Now that we've done so, we can log the ones we've initialized
        self.for_each_descriptor(|descriptor| {
            info!("Initialized driver '{}'", descriptor.driver.name())
        });
    }

    fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor)) {
        self.inner.lock(|inner| {
            inner
                .descriptors
                .iter()
                .filter_map(|x| x.as_ref())
                .for_each(f)
        })
    }
}

static DRIVER_MANAGER: DriverManager = DriverManager::new();

pub fn manager() -> &'static DriverManager {
    &DRIVER_MANAGER
}