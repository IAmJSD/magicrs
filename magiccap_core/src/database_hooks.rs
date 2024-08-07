// TODO: handle autoupdate and startup options.

// Defines hooks when a key is deleted.
pub fn on_delete(key: &str) {
    // If the key has the suffix '_hotkey', unregister the hotkey.
    if key.ends_with("_hotkey") {
        crate::hotkeys::drop_hotkey(key);

        // Reload the tray in case this is a OS that supports showing the modifier keys.
        crate::tray::load_tray();

        return;
    }

    // Handle if uploader_type is deleted and reload the tray if so.
    if key == "uploader_type" {
        crate::tray::load_tray()
    }
}

// Defines hooks when a key is set.
pub fn on_set(key: &str, value: &serde_json::Value) {
    // If the key has the suffix '_hotkey', register the hotkey.
    if key.ends_with("_hotkey") {
        if let serde_json::Value::String(hotkey) = value {
            crate::hotkeys::set_hotkey(key, hotkey);

            // Reload the tray in case this is a OS that supports showing the modifier keys.
            crate::tray::load_tray();
        }
        return;
    }

    // Handle if uploader_type is set and reload the tray if so.
    if key == "uploader_type" {
        crate::tray::load_tray()
    }
}

// Defines when a uploader is edited.
pub fn on_uploader_edit(_: &str) {
    // Update the tray in case the uploader is now enabled or disabled.
    crate::tray::load_tray()
}

// Defines many changes to the database.
pub fn on_bulk_changes() {
    // Reload the hotkeys in case those changed.
    crate::hotkeys::register_hotkeys();

    // Update the tray in case any items have changed.
    crate::tray::load_tray()
}
