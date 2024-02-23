use std::{fs, io::Read};

use base64::Engine;

pub fn proxy_fp(fp: &str) -> Result<Vec<u8>, String> {
    // Check if the extension is allowed.
    let ext = match fp.split('.').last() {
        Some(v) => v,
        None => return Err("File has no extension.".to_string()),
    };
    if !["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"].contains(&ext) {
        return Err("File extension is not allowed.".to_string());
    }
    let mime = "image/".to_string() + ext;

    // Check the file is under 5MB.
    let metadata = match fs::metadata(fp) {
        Ok(x) => x,
        Err(err) => return Err(format!("Error getting file metadata: {}", err)),
    };
    if metadata.len() > 5_000_000 {
        return Err("File is too large to proxy.".to_string());
    }

    // Read the file.
    let mut file = match fs::File::open(fp) {
        Ok(x) => x,
        Err(err) => return Err(format!("Error opening file: {}", err)),
    };
    let mut data = Vec::new();
    let data = match file.read_to_end(&mut data) {
        Ok(_) => data,
        Err(err) => return Err(format!("Error reading file: {}", err)),
    };

    // Return a data URI with the file.
    Ok(format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(data)).as_bytes().to_vec())
}
