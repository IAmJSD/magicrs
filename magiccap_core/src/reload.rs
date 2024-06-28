use crate::{statics::run_thread, tray::load_tray};

pub fn application_reload() {
    // Connect to the database.
    crate::database::connect();

    // Pre-load the textures in a thread.
    run_thread(|| crate::region_selector::preload_textures());

    // Load the tray.
    load_tray();
}
