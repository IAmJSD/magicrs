mod application_lock;
mod capture;
mod clipboard_actions;
mod config;
mod database;
mod reload;
mod tray;
mod uploaders;
mod notification;
mod mainthread;
mod statics;
mod unload;
mod utils;
mod region_selector;
mod temp_icon;

use application_lock::acquire_application_lock;
use config::open_config;
use mainthread::main_thread_async;

#[cfg(target_os = "macos")]
mod macos_delegate;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux_shared;

#[no_mangle]
pub unsafe extern "C" fn application_init() {
    // Handle the single instance lock.
    acquire_application_lock(|| {
        main_thread_async(|| open_config());
    });

    // Call the application init logic.
    #[cfg(target_os = "macos")]
    macos_delegate::application_init();
    #[cfg(target_os = "linux")]
    linux_shared::application_init();
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    compile_error!("Unsupported OS");
}

#[no_mangle]
pub unsafe extern "C" fn application_reload() {
    // On macOS or Linux, before we do anything, hydrate the application.
    #[cfg(target_os = "macos")]
    macos_delegate::application_hydrate();
    #[cfg(target_os = "linux")]
    linux_shared::application_hydrate();

    // Call the reload logic.
    reload::application_reload();
}
