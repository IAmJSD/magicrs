#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod other;
mod shared;

// On Linux, re-export the icond function.
#[cfg(target_os = "linux")]
pub use linux::icond;

// Re-export the icon handler.
#[cfg(target_os = "linux")]
pub use linux::IconHandler;
//#[cfg(not(target_os = "linux"))]
//pub use other::IconHandler;
