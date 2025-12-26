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

    // Setup glfw instance for region selector.
    crate::region_selector::setup_glfw_instance_for_region_selector();

    // If this is Linux, re-initialize the framebuffer daemon.
    #[cfg(target_os = "linux")]
    crate::framebufferd::init();
}
