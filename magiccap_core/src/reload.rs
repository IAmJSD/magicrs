use crate::{statics::run_thread, tray::load_tray};

pub fn application_reload() {
    // Connect to the database.
    crate::database::connect();

    // Pre-load the textures in a thread.
    run_thread(|| crate::region_selector::preload_textures());

    // Pre-unpack the frontend in a thread.
    run_thread(|| crate::config::pre_unpack_frontend());

    // Load the hotkeys.
    crate::hotkeys::register_hotkeys();

    // Load the tray.
    load_tray();
}
