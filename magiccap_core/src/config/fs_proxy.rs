use std::{fs, io::{BufReader, Cursor}};
use base64::Engine;
use image::GenericImageView;

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

    // Read the file into a image using the image crate.
    let file = match fs::File::open(fp) {
        Ok(x) => x,
        Err(err) => return Err(format!("Error opening file: {}", err)),
    };
    let fs_reader = BufReader::new(&file);
    let file_format = image::ImageFormat::from_extension(ext).unwrap();
    let file_format_clone = file_format.clone();
    let img = match image::load(fs_reader, file_format) {
        Ok(x) => x,
        Err(err) => return Err(format!("Error loading image: {}", err)),
    };

    // Shrinking the image to under 250x250 (preserving aspect ratio).
    let (width, height) = img.dimensions();
    let (new_width, new_height) = if width > height {
        (250, 250 * height / width)
    } else {
        (250 * width / height, 250)
    };
    let img = img.resize(new_width, new_height, image::imageops::FilterType::Nearest);

    // Encode the image to a data URI using the format specified by the mime type.
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    match img.write_to(&mut cursor, file_format_clone) {
        Ok(_) => (),
        Err(err) => return Err(format!("Error writing image to buffer: {}", err)),
    }

    // Return a data URI with the file.
    Ok(format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(cursor.into_inner())).as_bytes().to_vec())
}
