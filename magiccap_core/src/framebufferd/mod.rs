mod token;
mod shared;

use serde::Deserialize;
pub use token::get_token;

use crate::notification;

// Initialize the module.
pub fn init() {
    if let Err(_) = get_token() {
        notification::send_dialog_message(
            "MagicCap requires framebufferd to be running.",
        );
        std::process::exit(1);
    }
}

pub struct RGBAResult {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

fn encode_uri(uri: &str) -> String {
    urlencoding::encode(uri).to_string()
}

// Get the DRM RGBA capture.
pub fn get_drm_rgba_capture(fb_id: u32, device_path: String, vsync: bool) -> Option<RGBAResult> {
    let path = format!("/drm/rgba?fb_id={}&device_path={}&vsync={}",
        fb_id, encode_uri(&device_path), if vsync { "true" } else { "false" });
    let token = match get_token() {
        Ok(t) => match t {
            Some(tok) => tok,
            None => return None,
        }
        Err(_) => return None,
    };
    let (data, status, headers) = match shared::do_get_or_patch_request_to_socket(&path, false, Some(token)) {
        Some(r) => r,
        None => return None,
    };
    if status != 200 {
        eprintln!("Failed to get DRM RGBA capture, status code: {}", status);
        return None;
    }
    let width = match headers.get("x-width") {
        Some(w) => match w.to_str() {
            Ok(s) => match s.parse::<u32>() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to parse width from header: {}", e);
                    return None;
                }
            },
            Err(e) => {
                eprintln!("Failed to convert width header to str: {}", e);
                return None;
            }
        },
        None => {
            eprintln!("Width header not found");
            return None;
        }
    };
    let height = match headers.get("x-height") {
        Some(h) => match h.to_str() {
            Ok(s) => match s.parse::<u32>() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to parse height from header: {}", e);
                    return None;
                }
            },
            Err(e) => {
                eprintln!("Failed to convert height header to str: {}", e);
                return None;
            }
        },
        None => {
            eprintln!("Height header not found");
            return None;
        }
    };
    Some(RGBAResult {
        width,
        height,
        data,
    })
}

#[derive(Deserialize)]
pub struct Device {
    pub is_drm: bool,
    pub fb_id: Option<u32>,
    pub file_path: Option<String>,
    pub connector_name: Option<String>,
    pub x11_output: Option<String>,
}

// Lists all devices.
pub fn list_devices() -> Option<Vec<Device>> {
    let token = match get_token() {
        Ok(t) => match t {
            Some(tok) => tok,
            None => return None,
        }
        Err(_) => return None,
    };
    let (data, status, _) = match shared::do_get_or_patch_request_to_socket("/list", false, Some(token)) {
        Some(r) => r,
        None => return None,
    };
    if status != 200 {
        eprintln!("Failed to list devices, status code: {}", status);
        return None;
    }
    let devices: Vec<Device> = match serde_json::from_slice(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to parse devices JSON: {}", e);
            return None;
        }
    };
    Some(devices)
}
