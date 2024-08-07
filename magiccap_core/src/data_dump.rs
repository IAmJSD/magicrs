use crate::database::Capture;
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::Path,
};

static MAGIC_BYTES: &[u8] = b"MAGIC1";

#[derive(Deserialize, Serialize)]
struct DataDumpHeader {
    // Defines the number of captures that will follow.
    num_captures: u32,

    // Defines the configuration for the application.
    app_config: HashMap<String, serde_json::Value>,

    // Defines the configuration for custom uploaders.
    uploaders_config: HashMap<String, HashMap<String, serde_json::Value>>,

    // Defines the user directory.
    user_dir: String,

    // Defines if this is Windows or not.
    is_windows: bool,
}

#[derive(Deserialize, Serialize)]
struct CaptureHeader {
    pub created_at: String,
    pub success: bool,
    pub filename: String,
    pub url: Option<String>,

    // This option defines if data was saved to the disk and if so how much follows.
    pub saved_bytes: Option<u64>,
}

// Create a data dump from MagicCap's database. The option is an error if set.
pub fn dump_data(fp: String) -> Option<String> {
    // Create the file for writing.
    let mut file = match fs::File::create(fp) {
        Ok(f) => f,
        Err(e) => return Some(e.to_string()),
    };

    // Read all the captures.
    let captures = crate::database::get_captures();

    // Get all the application configuration options.
    let app_config = crate::database::get_all_config_options();

    // Get all the uploaders configuration options.
    let uploaders_config = crate::database::get_all_uploaders_config_options();

    // Create the header and turn it into JSON.
    let header = DataDumpHeader {
        num_captures: captures.len() as u32,
        app_config,
        uploaders_config,
        user_dir: home::home_dir().unwrap().to_string_lossy().to_string(),
        is_windows: cfg!(target_os = "windows"),
    };
    let header = match serde_json::to_vec(&header) {
        Ok(h) => h,
        Err(e) => return Some(e.to_string()),
    };

    // Write the magic bytes and the length of the decompressed header.
    let mut chunk = [0; 10];
    chunk[0..6].copy_from_slice(MAGIC_BYTES);
    chunk[6..10].copy_from_slice(&(header.len() as u32).to_le_bytes());
    match file.write_all(&chunk) {
        Ok(_) => (),
        Err(e) => return Some(e.to_string()),
    }

    // Create a gzip compressor.
    let mut compressor = flate2::write::GzEncoder::new(file, flate2::Compression::default());

    // Write the header.
    match compressor.write_all(&header) {
        Ok(_) => (),
        Err(e) => return Some(e.to_string()),
    }

    // Go through all the captures.
    let mut io_buffer = [0; 4096];
    for capture in captures {
        // Get a file handle and bytes count if applicable.
        let file_reader_and_len = match capture.file_path {
            Some(fp) => {
                // Stat the file to get the length.
                match fs::metadata(&fp) {
                    Ok(m) => {
                        // Get the length of the file.
                        let len = m.len();

                        // Open the file for reading.
                        match fs::File::open(fp) {
                            Ok(f) => Some((f, len)),
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                }
            }
            None => {
                // If there is no file path, we don't need to do anything.
                None
            }
        };

        // Create the header and turn it into JSON.
        let header = CaptureHeader {
            created_at: capture.created_at,
            success: capture.success,
            filename: capture.filename,
            url: capture.url,
            saved_bytes: file_reader_and_len.as_ref().map(|(_, l)| l.clone()),
        };
        let header = match serde_json::to_vec(&header) {
            Ok(h) => h,
            Err(e) => return Some(e.to_string()),
        };

        // Write the length of the header.
        match compressor.write_all(&(header.len() as u32).to_le_bytes()) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        }

        // Write the header.
        match compressor.write_all(&header) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        }

        // If there is a file path, write the file.
        if let Some((mut reader, _)) = file_reader_and_len {
            // Read the file and write it to the compressor.
            loop {
                match reader.read(&mut io_buffer) {
                    Ok(0) => break,
                    Ok(n) => match compressor.write_all(&io_buffer[0..n]) {
                        Ok(_) => (),
                        Err(e) => return Some(e.to_string()),
                    },
                    Err(e) => return Some(e.to_string()),
                }
            }
        }
    }

    // Finalize the compressor.
    match compressor.finish() {
        Ok(_) => (),
        Err(e) => return Some(e.to_string()),
    }

    // Return no errors.
    None
}

// Turns the config option into a path.
fn config2path(config: Option<&str>, header: &DataDumpHeader) -> Option<String> {
    match config {
        Some(c) => {
            // Replace any instance of the user directory with the user directory on our system.
            let c = c.replace(header.user_dir.as_str(), &header.user_dir);

            // Handle if the path type is different.
            if cfg!(target_os = "windows") {
                if header.is_windows {
                    Some(c)
                } else {
                    Some(c.replace("\\", "/"))
                }
            } else {
                if header.is_windows {
                    Some(c.replace("\\", "/"))
                } else {
                    Some(c)
                }
            }
        }
        None => None,
    }
}

// Load the data dump into MagicCap's database. The option is an error if set.
pub fn load_data(fp: String) -> Option<String> {
    // Open the file for reading.
    let mut file = match fs::File::open(fp) {
        Ok(f) => f,
        Err(e) => {
            return Some(format!(
                "failed to open file for reading: {}",
                e.to_string(),
            ))
        }
    };

    // Go ahead and make sure it starts with the magic bytes and u32 length of the decompressed header.
    let mut magic_bytes_and_len = [0; 10];
    match file.read_exact(&mut magic_bytes_and_len) {
        Ok(_) => (),
        Err(e) => {
            return Some(format!(
                "failed to read magic bytes and length of decompressed header: {}",
                e.to_string(),
            ))
        }
    }

    // Check the magic bytes are at the start.
    if &magic_bytes_and_len[0..6] != MAGIC_BYTES {
        return Some("Magic bytes not found at start of file.".to_string());
    }

    // Get the length of the decompressed header.
    let header_decomp_len = u32::from_le_bytes([
        magic_bytes_and_len[6],
        magic_bytes_and_len[7],
        magic_bytes_and_len[8],
        magic_bytes_and_len[9],
    ]);

    // Make a gunzip decompressor.
    let mut decompressor = flate2::read::GzDecoder::new(file);

    // Read the header.
    let mut header = Vec::new();
    match Vec::try_reserve_exact(&mut header, header_decomp_len as usize) {
        Ok(_) => (),
        Err(e) => return Some(e.to_string()),
    }
    unsafe { header.set_len(header_decomp_len as usize) };
    match decompressor.read_exact(&mut header) {
        Ok(_) => (),
        Err(e) => return Some(e.to_string()),
    }

    // Deserialize the header.
    let header: DataDumpHeader = match serde_json::from_slice(&header) {
        Ok(h) => h,
        Err(e) => return Some(e.to_string()),
    };

    // Get the save path if it is defined.
    let folder_path = match header.app_config.get("folder_path") {
        Some(v) => match v.as_str() {
            Some(s) => Some(s),
            None => return Some("folder_path is not a string.".to_string()),
        },
        None => None,
    };
    let mut folder_path = config2path(folder_path, &header);

    // Defines if mkdir all was already called.
    let mut mkdir_all_called = false;

    // Process the captures.
    let mut captures: Vec<Capture> = Vec::new();
    match Vec::try_reserve_exact(&mut captures, header.num_captures as usize) {
        Ok(_) => (),
        Err(e) => return Some(e.to_string()),
    }
    let mut io_buffer = [0; 4096];
    for i in 0..header.num_captures {
        // Read the capture header length.
        let mut header_len_bytes = [0; 4];
        match decompressor.read_exact(&mut header_len_bytes) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        }

        // Read the capture header.
        let header_len = u32::from_le_bytes(header_len_bytes);
        let mut header = Vec::new();
        match Vec::try_reserve_exact(&mut header, header_len as usize) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        }
        unsafe { header.set_len(header_len as usize) };
        match decompressor.read_exact(&mut header) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        }

        // Deserialize the capture header.
        let header: CaptureHeader = match serde_json::from_slice(&header) {
            Ok(h) => h,
            Err(e) => return Some(e.to_string()),
        };

        if let Some(capture_len) = header.saved_bytes {
            // Get the local capture path.
            let local_folder_path = match folder_path.clone() {
                Some(p) => {
                    if !mkdir_all_called {
                        match fs::create_dir_all(&p) {
                            Ok(_) => (),
                            Err(e) => return Some(e.to_string()),
                        }
                        mkdir_all_called = true;
                    }
                    p
                }
                None => {
                    // Ensure ~/Pictures/MagicCap exists.
                    let mut homedir = home::home_dir().unwrap();
                    homedir.push("Pictures");
                    homedir.push("MagicCap");
                    match fs::create_dir_all(&homedir) {
                        Ok(_) => (),
                        Err(e) => return Some(e.to_string()),
                    }

                    // Set it as the folder path.
                    folder_path = Some(homedir.to_str().unwrap().to_string());

                    // Return the path.
                    homedir.to_str().unwrap().to_string()
                }
            };
            let local_capture_path = Path::new(&local_folder_path).join(&header.filename);

            // Make sure the local capture path contains the folder path to prevent directory traversal.
            if !local_capture_path.starts_with(&local_folder_path) {
                return Some("Local capture path is outside of the folder path.".to_string());
            }

            // Write the capture into the file.
            let mut capture_file = match fs::File::create(&local_capture_path) {
                Ok(f) => f,
                Err(e) => return Some(e.to_string()),
            };
            let mut bytes_written = 0;
            while bytes_written < capture_len {
                let bytes_to_read = cmp::min(io_buffer.len() as u64, capture_len - bytes_written);
                let bytes_read = match decompressor.read(&mut io_buffer[0..bytes_to_read as usize])
                {
                    Ok(b) => b,
                    Err(e) => return Some(e.to_string()),
                };
                match capture_file.write_all(&io_buffer[0..bytes_read]) {
                    Ok(_) => (),
                    Err(e) => return Some(e.to_string()),
                }
                bytes_written += bytes_read as u64;
            }

            // Add the capture to the captures list.
            captures.push(Capture {
                id: i as i64,
                created_at: header.created_at,
                success: header.success,
                filename: header.filename,
                url: header.url,
                file_path: Some(local_capture_path.to_str().unwrap().to_string()),
            });
        } else {
            // This capture was not saved to the disk.
            captures.push(Capture {
                id: i as i64,
                created_at: header.created_at,
                success: header.success,
                filename: header.filename,
                url: header.url,
                file_path: None,
            });
        }
    }

    // Write the header and captures to the database.
    crate::database::rewrite(header.app_config, header.uploaders_config, captures);

    // Return no errors.
    None
}
