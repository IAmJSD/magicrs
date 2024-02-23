use crate::tray::load_tray;

pub fn application_reload() {
    // Connect to the database.
    crate::database::connect();

    // Load the tray.
    load_tray();
}
