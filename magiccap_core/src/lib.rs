#[cfg(target_os = "windows")]
extern crate native_windows_gui as nwg;

mod capture;
mod clipboard_actions;
mod config;
mod data_dump;
mod database;
mod database_hooks;
mod hotkeys;
mod mainthread;
mod notification;
mod ocr;
mod region_selector;
mod reload;
mod search_indexing;
mod statics;
mod temp_icon;
mod tray;
mod unload;
mod uploaders;
mod utils;
mod video_capture;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
mod macos_delegate;

#[cfg(target_os = "linux")]
mod linux_shared;

#[cfg(target_os = "windows")]
mod windows_shared;

#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn application_init() {
    macos_delegate::application_init()
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn application_init() {
    linux_shared::application_init()
}

#[cfg(target_os = "windows")]
#[no_mangle]
pub unsafe extern "C" fn application_init() {
    windows_shared::application_init()
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
