use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::{collections::HashMap, str::FromStr, sync::Mutex};

use crate::{capture, database, statics::run_thread};

pub struct HotkeyWrapper {
    ghm: GlobalHotKeyManager,
    hotkeys: HashMap<String, HotKey>,
    hash_id_to_magiccap_id: &'static Mutex<HashMap<u32, String>>,
}

fn magiccap_id_hit(id: &str) {
    let id_clone = id.to_string();
    run_thread(move || match id_clone.as_str() {
        "region_hotkey" => capture::region_capture(),
        "fullscreen_hotkey" => capture::fullscreen_capture(),
        "gif_hotkey" => capture::gif_capture(),
        "video_hotkey" => capture::video_capture(),
        "clipboard_hotkey" => capture::clipboard_capture(),
        _ => {}
    })
}

static DATABASE_KEYS: &[&str] = &[
    "region_hotkey",
    "fullscreen_hotkey",
    "gif_hotkey",
    "video_hotkey",
    "clipboard_hotkey",
];

impl HotkeyWrapper {
    pub fn new() -> Self {
        let hi2mi = Box::leak(Box::new(Mutex::new(HashMap::new())));
        let instance = Self {
            ghm: GlobalHotKeyManager::new().unwrap(),
            hotkeys: HashMap::new(),
            hash_id_to_magiccap_id: hi2mi,
        };
        let receiver = GlobalHotKeyEvent::receiver();
        std::thread::spawn(|| loop {
            if let Ok(event) = receiver.try_recv() {
                if event.state() == HotKeyState::Pressed {
                    let id = event.id();
                    let id_lock = hi2mi.lock().unwrap();
                    if let Some(id) = id_lock.get(&id) {
                        magiccap_id_hit(id);
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        });
        instance
    }

    fn register_by_id(&mut self, id: &str, hotkey: &str) {
        match self.hotkeys.remove(id) {
            Some(v) => self.ghm.unregister(v).unwrap(),
            None => {}
        };
        let hotkey = match HotKey::from_str(hotkey) {
            Ok(h) => h,
            Err(_) => return,
        };
        if let Err(_) = self.ghm.register(hotkey) {
            return;
        }
        let mut lock = self.hash_id_to_magiccap_id.lock().unwrap();
        lock.insert(hotkey.id, id.to_string());
    }

    fn unregister_by_id(&mut self, id: &str) {
        let val = match self.hotkeys.remove(id) {
            Some(v) => v,
            None => return,
        };
        self.ghm.unregister(val).unwrap()
    }

    fn unregister_all(&mut self) {
        let values = self.hotkeys.values_mut().map(|v| *v).collect::<Vec<_>>();
        self.hotkeys = HashMap::new();
        self.ghm.unregister_all(values.as_slice()).unwrap()
    }
}

fn hr_ref() -> &'static mut HotkeyWrapper {
    #[cfg(target_os = "linux")]
    return &mut crate::linux_shared::app().hotkey_wrapper;
    #[cfg(target_os = "windows")]
    return &mut crate::windows_shared::app().hotkey_wrapper;
}

// Set a hotkey by its ID.
pub fn set_hotkey(id: &str, hotkey: &str) {
    hr_ref().register_by_id(id, hotkey)
}

// Handles (re-)registering hotkeys and their callbacks.
pub fn register_hotkeys() {
    let r = hr_ref();
    r.unregister_all();
    for db_key in DATABASE_KEYS {
        let val = database::get_config_option(db_key);
        if let Some(val_json) = val {
            let val_str = match val_json.as_str() {
                Some(s) => s,
                None => return,
            };
            r.register_by_id(db_key, val_str);
        }
    }
}

// Drop a hotkey by its ID.
pub fn drop_hotkey(id: &str) {
    hr_ref().unregister_by_id(id);
}

// Drops all hotkeys.
pub fn drop_all_hotkeys() {
    hr_ref().unregister_all();
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
