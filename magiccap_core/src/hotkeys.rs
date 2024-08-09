use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    capture, database,
    mainthread::{main_thread_async, main_thread_sync},
    statics::run_thread,
};

pub struct HotkeyWrapper {
    ghm: GlobalHotKeyManager,
    hotkeys: HashMap<String, HotKey>,
    hash_id_to_magiccap_id: &'static Mutex<HashMap<u32, String>>,
}

static WAIT_COUNT: AtomicU64 = AtomicU64::new(0);

fn magiccap_id_hit(id: &str) {
    if WAIT_COUNT.load(Ordering::Relaxed) != 0 {
        return;
    }
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

#[cfg(target_os = "linux")]
pub struct HotkeyCapture {
    // These are options so they can be taken without a copy.
    press_signal_handler_id: Option<glib::SignalHandlerId>,
    release_signal_handler_id: Option<glib::SignalHandlerId>,
}

#[cfg(target_os = "linux")]
fn send_hotkey_capture(hotkey: String) {
    use crate::{linux_shared::app, mainthread::main_thread_async};
    use base64::Engine;
    use webkit2gtk::WebViewExt;

    // Base64 encode the hotkey.
    let hotkey_base64 = base64::prelude::BASE64_STANDARD.encode(hotkey);

    // Since we need the main thread on Linux, we push a async main thread task here.
    main_thread_async(move || {
        let read_ref = app().webview.read().unwrap();
        if let Some(webview) = read_ref.as_ref() {
            // Yes, this is SUPER brutal.
            webview.value.run_javascript(
                &format!(
                    "window.bridgeResponse(-2, '{}');", // see persistentHandlers in frontend/src/bridge/implementation.ts
                    hotkey_base64
                ),
                None::<&gio::Cancellable>,
                |_| {},
            );
        }
    });
}

#[cfg(target_os = "linux")]
fn process_keys(m: &mut Vec<String>) {
    // Remove any _L or _R from values.
    for val in m.into_iter() {
        if val.ends_with("_L") || val.ends_with("_R") {
            let mut chars = val.chars();
            chars.next_back();
            chars.next_back();
            *val = chars.as_str().to_string();
        }
    }
}

#[cfg(target_os = "linux")]
fn send_vec_state(
    vec_guard: std::sync::MutexGuard<Vec<crate::linux_shared::FakeSend<gdk::EventKey>>>,
) {
    // Get the down keys.
    let mut string_uniques: HashMap<String, ()> = HashMap::new();
    for item in vec_guard.as_slice() {
        let key = item.value.hardware_keycode();
        let display = gdk::Display::default().unwrap();
        let entries = gdk::Keymap::for_display(&display)
            .unwrap()
            .entries_for_keycode(key as u32);
        let first_entry = entries.first().unwrap().1;
        let key = gdk::keys::Key::from(first_entry);
        if let Some(key_name) = key.name() {
            string_uniques.insert(key_name.to_string(), ());
        }
    }
    let mut keys = string_uniques.into_keys().collect::<Vec<_>>();
    if keys.len() == 0 {
        // No keys down right now.
        return;
    }
    process_keys(&mut keys);
    keys.sort();

    // Send the hotkeys to the browser.
    send_hotkey_capture(keys.join("+"))
}

impl HotkeyCapture {
    #[cfg(target_os = "linux")]
    pub fn new() -> Self {
        use crate::linux_shared::{app, FakeSend};
        use gtk::prelude::WidgetExt;

        WAIT_COUNT.fetch_add(1, Ordering::Relaxed);

        let down_keys = Arc::new(Mutex::new(Vec::new()));
        let down1 = down_keys.clone();
        let down2 = down_keys.clone();
        let (press_signal_handler_id, release_signal_handler_id) = main_thread_sync(move || {
            let wv_lock = app().webview.read().unwrap();
            let wv_ref = wv_lock.as_ref().unwrap();

            let press_id = wv_ref.value.connect_key_press_event(move |_, key| {
                let arc_clone = down1.clone();
                let mut vec_guard = arc_clone.lock().unwrap();
                vec_guard.push(FakeSend { value: key.clone() });
                send_vec_state(vec_guard);
                glib::Propagation::Stop
            });

            let release_id = wv_ref.value.connect_key_release_event(move |_, key| {
                let arc_clone = down2.clone();
                let mut v = arc_clone.lock().unwrap();
                if let Some(i) = v
                    .iter()
                    .position(|x| x.value.hardware_keycode() == key.hardware_keycode())
                {
                    v.remove(i);
                }
                send_vec_state(v);
                glib::Propagation::Stop
            });

            (FakeSend { value: press_id }, FakeSend { value: release_id })
        });

        Self {
            press_signal_handler_id: Some(press_signal_handler_id.value),
            release_signal_handler_id: Some(release_signal_handler_id.value),
        }
    }
}

impl Drop for HotkeyCapture {
    fn drop(&mut self) {
        // Subtract from the wait count.
        WAIT_COUNT.fetch_sub(1, Ordering::Relaxed);

        // On Linux, drop both signal handlers on the main thread.
        #[cfg(target_os = "linux")]
        {
            let press_signal_handler_id = self.press_signal_handler_id.take().unwrap();
            let release_signal_handler_id = self.release_signal_handler_id.take().unwrap();
            main_thread_async(move || {
                use crate::linux_shared::app;
                use glib::ObjectExt;

                let wv_lock = app().webview.read().unwrap();
                if let Some(wv_ref) = wv_lock.as_ref() {
                    wv_ref.value.disconnect(press_signal_handler_id);
                    wv_ref.value.disconnect(release_signal_handler_id);
                }
            })
        }

        // TODO: Drop on other platforms!
    }
}
