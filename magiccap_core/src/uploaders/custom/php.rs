use std::{collections::HashMap, path::PathBuf};
use crate::statics::CONFIG_FOLDER;
use super::php_bootstrapping::{setup_php_binary, validate_php_metadata};

fn random_string() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut string = String::new();
    for _ in 0..10 {
        string.push(rng.gen_range(b'a'..=b'z') as char);
    }
    string
}

// Handle generating the PHP ini file.
fn php_ini_gen(temp_folder: &PathBuf) -> String {
    return "disable_functions=exec,passthru,shell_exec,system,proc_open,popen,curl_exec,curl_multi_exec,parse_ini_file,show_source
allow_url_include=off
open_basedir=".to_string() + temp_folder.to_str().unwrap() + "/content";
}

// Handles copying the data from the perma_data_folder to the data_folder. Returns an error if one occurs.
async fn copy_data(perma_data_folder: &str, data_folder: &PathBuf) -> Result<(), String> {
    use tokio::fs;

    // Get the directory contents.
    let mut dir = match fs::read_dir(perma_data_folder).await {
        Ok(d) => d,
        Err(e) => return Err(e.to_string()),
    };

    // Iterate over the directory contents.
    for entry in dir.next_entry().await.map_err(|e| e.to_string())? {
        // Get the entry path.
        let entry_path = entry.path();

        // Get the metadata.
        let metadata = match fs::metadata(&entry_path).await {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };

        // If the entry is a directory, create it.
        if metadata.is_dir() {
            match fs::create_dir(data_folder.join(entry.file_name())).await {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            };
            continue;
        }

        // If the entry is a file, copy it.
        match fs::copy(&entry_path, data_folder.join(entry.file_name())).await {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        };
    }

    // Return no error.
    return Ok(());
}

// Handle building the PHP temporary folder.
fn build_php_temp_folder(
    perma_data_folder: &str, php_code: &str, filename: &str,
    config: HashMap<String, serde_json::Value>,
    mut reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<PathBuf, String> {
    // Create the temporary folder.
    let temp_folder = std::env::temp_dir().join(
        "magiccap-php-".to_string() + &random_string(),
    );
    match std::fs::create_dir(&temp_folder) {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    };

    // Create the PHP script.
    let php_script = temp_folder.clone().join("script.php");
    match std::fs::write(&php_script, php_code) {
        Ok(_) => (),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_folder);
            return Err(e.to_string());
        },
    };

    // Create the PHP ini file.
    let php_ini = temp_folder.join("php.ini");
    match std::fs::write(&php_ini, php_ini_gen(&temp_folder)) {
        Ok(_) => (),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_folder);
            return Err(e.to_string());
        },
    };

    // Create the content and data folders.
    let content_folder = temp_folder.join("content");
    match std::fs::create_dir(&content_folder) {
        Ok(_) => (),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_folder);
            return Err(e.to_string());
        },
    };

    // Write config.json to the content folder.
    let config_json = content_folder.join("config.json");
    match std::fs::write(&config_json, serde_json::to_string(&config).unwrap()) {
        Ok(_) => (),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_folder);
            return Err(e.to_string());
        },
    };

    // Try to copy the permanent data folder to the temporary data folder.
    let data_folder = content_folder.clone().join("data");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(copy_data(perma_data_folder, &data_folder))?;

    // Create the screenshot file.
    let screenshot_path = content_folder.join(filename);
    let mut file = match std::fs::File::create(&screenshot_path) {
        Ok(f) => f,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_folder);
            return Err(e.to_string());
        },
    };
    match std::io::copy(&mut reader, &mut file) {
        Ok(_) => (),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_folder);
            return Err(e.to_string());
        },
    };

    // Return the temporary folder.
    return Ok(temp_folder);
}

// Handles cleaning up the PHP temporary folder. There being a String means an error occurred.
fn php_cleanup(temp_folder: PathBuf, perma_data_folder: PathBuf) -> Option<String> {
    let data_dir = temp_folder.join("content").join("data");
    if data_dir.exists() {
        // Try removing the perma_data_folder.
        match std::fs::remove_dir_all(&perma_data_folder) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        };

        // Move the data_dir to the perma_data_folder.
        match std::fs::rename(data_dir, perma_data_folder) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        };
    }
    let _ = std::fs::remove_dir_all(&temp_folder);
    return None;
}

// Handle the function that does the PHP upload.
pub fn php(
    uploader_id: &str, php_code: &str,
    config: HashMap<String, serde_json::Value>,
    filename: &str, reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Get the PHP binary path.
    let php_path = CONFIG_FOLDER.join("php");
    let php_path = match php_path.metadata() {
        Ok(metadata) => {
            match validate_php_metadata(php_path, metadata) {
                Ok(s) => s,
                Err(e) => return Err(e),
            }
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                match setup_php_binary(php_path) {
                    Ok(s) => s,
                    Err(e) => return Err(e),
                }
            } else {
                return Err(e.to_string());
            }
        }
    };

    // Defines the permanant folder for data storage.
    let perma_data_folder = CONFIG_FOLDER.join("custom_uploaders").join(uploader_id);

    // MkdirAll if it doesn't exist.
    if !perma_data_folder.exists() {
        match std::fs::create_dir_all(&perma_data_folder) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        }
    }

    // Build the PHP temporary folder.
    let temp_folder = match build_php_temp_folder(
        perma_data_folder.to_str().unwrap(), php_code, filename, config, reader,
    ) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    // Defines all of the paths we now know.
    let php_script = temp_folder.join("script.php");
    let php_ini = temp_folder.join("php.ini");
    let content_folder = temp_folder.join("content");
    let data_folder = content_folder.join("data");
    let screenshot_path = content_folder.join(filename);

    // Execute the PHP script and get the output.
    let output = std::process::Command::new(php_path)
        .arg("-c")
        .arg(php_ini)
        .arg(php_script)
        .env("DATA_DIR", data_folder)
        .env("SCREENSHOT_PATH", screenshot_path)
        .output();

    // In all cases, do the cleanup.
    if let Some(e) = php_cleanup(temp_folder, perma_data_folder) {
        return Err(e);
    }

    // Unwrap the result.
    let output = match output {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };

    // Check if the PHP script ran successfully.
    if !output.status.success() {
        return Err(format!(
            "The PHP script failed with the following output: {}",
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    // Get the output as a string and ensure it is a valid URL.
    let output_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };
    let url = match uriparse::URI::try_from(output_str.trim()) {
        Ok(u) => u,
        Err(e) => return Err(e.to_string()),
    };

    // Return the URL.
    Ok(url.to_string())
}
