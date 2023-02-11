// Load the appropriate arch module
#[cfg(target_arch="aarch64")]
#[path = "./aarch64/mod.rs"]
mod aarch64;

#[cfg(target_arch="aarch64")]
pub use aarch64::*;