use crate::{database, mainthread, notification};
use serde_json::Value;

pub struct CaptureFile {
    #[allow(dead_code)] // Only dead on some platforms.
    pub file_name: String,
    pub content: Vec<u8>,
}

static DEFAULT: &str = "content";

#[cfg(target_os = "linux")]
fn write_clipboard_text(s: String) {
    mainthread::main_thread_sync(|| {
        // TODO: wayland
        let clipboard = gtk::Clipboard::default(&gdk::Display::default().unwrap()).unwrap();
        clipboard.set_text(s.as_str());
    })
}

#[cfg(target_os = "macos")]
fn write_clipboard_bytes(content: CaptureFile) {
    unsafe {
        use crate::macos::copy_file_to_clipboard;
        use std::ffi::CString;

        let mut fp_cstr = CString::new("").unwrap();
        let fp_ptr = match file_path {
            Some(fp) => {
                fp_cstr = CString::new(fp).unwrap();
                fp_cstr.as_ptr()
            }
            None => std::ptr::null(),
        };

        let filename_cstr = CString::new(content.file_name).unwrap();
        let data_len = content.content.len();

        copy_file_to_clipboard(
            fp_ptr,
            filename_cstr.as_ptr(),
            content.content.as_ptr(),
            data_len as usize,
        );
        drop(fp_cstr);
        drop(filename_cstr);
    }
}

#[cfg(target_os = "linux")]
fn write_clipboard_bytes(content: CaptureFile) {
    use gdk::prelude::PixbufLoaderExt;

    mainthread::main_thread_sync(|| {
        // TODO: wayland
        let clipboard = gtk::Clipboard::default(&gdk::Display::default().unwrap()).unwrap();
        let pixbuf_loader = gtk::gdk_pixbuf::PixbufLoader::new();
        let fake_static_slice =
            unsafe { std::mem::transmute::<&_, &'static _>(content.content.as_slice()) };
        match pixbuf_loader.write_bytes(&glib::Bytes::from_static(fake_static_slice)) {
            Ok(_) => {}
            Err(_) => return,
        };
        let pixbuf = match pixbuf_loader.pixbuf() {
            Some(p) => p,
            None => return,
        };
        pixbuf_loader.close().unwrap();
        clipboard.set_image(&pixbuf);
        drop(content.content);
    })
}

pub fn handle_clipboard_action(
    file_path: Option<&str>,
    url: Option<&str>,
    content: Option<CaptureFile>,
) {
    // Figure out the clipboard action.
    let value_scratch: Value;
    let action = match database::get_config_option("clipboard_action") {
        Some(action) => {
            value_scratch = action;
            match value_scratch.as_str() {
                Some(action) => action,
                None => {
                    // Error here.
                    notification::send_dialog_message(
                        "The clipboard action is not a string. Please file a bug report!",
                    );
                    return;
                }
            }
        }
        None => DEFAULT,
    };
    match action {
        "url" => {
            if let Some(url) = url {
                write_clipboard_text(url.to_string());
                return;
            }
        }
        "file_path" => {
            if let Some(file_path) = file_path {
                write_clipboard_text(file_path.to_string());
                return;
            }
        }
        "content" => (),

        "none" => return,
        _ => return,
    };

    // If any of the mechanisms specified do not yield anything and the content is not empty, use it.
    if let Some(content) = content {
        write_clipboard_bytes(content);
    }
}
