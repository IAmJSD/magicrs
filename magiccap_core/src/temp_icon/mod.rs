mod shared;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod other;

// On Linux, re-export the icond function.
#[cfg(target_os = "linux")]
pub use linux::icond;
