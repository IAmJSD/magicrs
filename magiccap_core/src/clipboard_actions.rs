use crate::{database, notification};
use copypasta::{ClipboardContext, ClipboardProvider};
use serde_json::Value;

pub struct CaptureFile {
    pub file_name: String,
    pub content: Vec<u8>,
}

static DEFAULT: &str = "content";

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
                let mut ctx = ClipboardContext::new().unwrap();
                ctx.set_contents(url.to_string()).unwrap();
                return;
            }
        }
        "file_path" => {
            if let Some(file_path) = file_path {
                let mut ctx = ClipboardContext::new().unwrap();
                ctx.set_contents(file_path.to_string()).unwrap();
                return;
            }
        }
        "content" => (),

        "none" => return,
        _ => return,
    };

    // If any of the mechanisms specified do not yield anything and the content is not empty, use it.
    if let Some(content) = content {
        // On macOS, call our Obj-C layer to handle the clipboard.
        #[cfg(target_os = "macos")]
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
}
