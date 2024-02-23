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

#[cfg(target_os = "macos")]
mod macos_delegate;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux_shared;

#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn application_init() { macos_delegate::application_init() }

#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn application_init() { linux_shared::application_init() }

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
