// Set a hotkey by its ID.
pub fn set_hotkey(id: &str, hotkey: &str) {
    // TODO
}

// Handles (re-)registering hotkeys and their callbacks.
pub fn register_hotkeys() {
    // TODO
}

// Drop a hotkey by its ID.
pub fn drop_hotkey(id: &str) {
    // TODO
}

pub struct HotkeyCapture {}

impl HotkeyCapture {
    pub fn new() -> Self {
        // TODO
        HotkeyCapture {}
    }
}

impl Drop for HotkeyCapture {
    fn drop(&mut self) {
        // TODO
    }
}
