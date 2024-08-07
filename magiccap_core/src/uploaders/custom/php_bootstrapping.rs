use once_cell::sync::Lazy;
use serde::Deserialize;
use sha2::Digest;
use std::{collections::HashMap, path::PathBuf};

// Defines artifact information.
#[derive(Deserialize)]
struct ArtifactInfo {
    url: String,
    sha256: String,
    zip: bool,
}

// Get the artifact information from a static file.
static ARTIFACT_BLOB: Lazy<HashMap<String, ArtifactInfo>> = Lazy::new(|| {
    let artifacts_bytes = include_bytes!("../../../../php_artifacts.json");
    serde_json::from_slice(artifacts_bytes).unwrap()
});

// Gets the artifact information for the OS we are running on.
fn get_artifact_info() -> Result<&'static ArtifactInfo, String> {
    // Get the OS and architecture.
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // Get the key for the artifact information.
    let key = if arch == "aarch64" {
        format!("php-{}-arm64", os)
    } else {
        format!("php-{}-{}", os, arch)
    };

    // Get the artifact information.
    let artifact_info = match ARTIFACT_BLOB.get(key.as_str()) {
        Some(a) => a,
        None => {
            return Err(
                "No PHP artifact information found for the current OS/architecture".to_string(),
            )
        }
    };

    // Return the artifact information.
    Ok(artifact_info)
}

// Check the integrity of a PHP folder.
fn check_php_folder(path: &PathBuf, sha256: &'static str) -> Result<bool, String> {
    // Get the directory contents recursively.
    let items = walkdir::WalkDir::new(path).into_iter().collect::<Vec<_>>();

    // Hash the files.
    let mut hash_results: Vec<[String; 2]> = Vec::with_capacity(items.len());
    for item in items {
        // Unwrap the item.
        let item = match item {
            Ok(i) => i,
            Err(e) => return Err(e.to_string()),
        };

        // If this is a file, load/hash the contents.
        let mut is_file = true;
        let file_hash = if item.file_type().is_file() {
            // Read the file.
            let file = match std::fs::read(item.path()) {
                Ok(f) => f,
                Err(e) => return Err(e.to_string()),
            };

            // Hash the file.
            let hash = sha2::Sha256::digest(&file);

            // Encode the hash as a hex string.
            hash.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        } else {
            is_file = false;
            "".to_string()
        };

        // Get the normalised path with the root as the relative start.
        let path = item
            .path()
            .strip_prefix(path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut path = path.replace("\\", "/");
        if !is_file && !path.ends_with("/") {
            path.push_str("/");
        }
        hash_results.push([path, file_hash]);
    }

    // Sort the results by the file path.
    hash_results.sort_by(|a, b| a[0].cmp(&b[0]));

    // Hash the results as a JSON array.
    let mut hasher = sha2::Sha256::new();
    hasher.update(serde_json::to_string(&hash_results).unwrap());
    let hash = hasher.finalize();

    // Encode the hash as a hex string.
    let hash_str = hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // Return if they are equal.
    Ok(hash_str == sha256)
}

// Check the integrity of a PHP binary.
fn check_php_bin(path: &PathBuf, sha256: &'static str) -> Result<bool, String> {
    use sha2::{Digest, Sha256};

    // Read the file.
    let file = match std::fs::read(path) {
        Ok(f) => f,
        Err(e) => return Err(e.to_string()),
    };

    // Hash the file.
    let mut hasher = Sha256::new();
    hasher.update(&file);
    let hash = hasher.finalize();

    // Encode the hash as a hex string.
    let hash_str = hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // Return if they are equal.
    Ok(hash_str == sha256)
}

// Checks the integrity of the PHP path. The outer Result is for an error that we should immediately return
// to the user and not proceed with, and the inner Result is for the artifact information. If the inner Result
// is a PathBuf, that is the path to the PHP binary that should be used. If the inner Result is the artifact
// information, that means the integrity check failed and the user should be prompted to redownload the PHP binary.
fn check_php_integrity(path: &PathBuf) -> Result<Result<PathBuf, &'static ArtifactInfo>, String> {
    let artifact = match get_artifact_info() {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    let integrity_check_passed = match if artifact.zip {
        check_php_folder(path, &artifact.sha256)
    } else {
        check_php_bin(path, &artifact.sha256)
    } {
        Ok(b) => b,
        Err(e) => return Err(e),
    };

    if integrity_check_passed {
        let php_path = if artifact.zip {
            path.join(if cfg!(windows) { "php.exe" } else { "php" })
        } else {
            path.clone()
        };
        Ok(Ok(php_path))
    } else {
        Ok(Err(artifact))
    }
}

// Extracts PHP from a ZIP archive into the path specified. Returns an error if one occurs.
fn extract_php(bytes: Vec<u8>, path: &PathBuf) -> Option<String> {
    // Create a ZIP archive reader.
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return Some(e.to_string()),
    };

    // Extract the ZIP archive.
    for i in 0..archive.len() {
        // Get the file.
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => return Some(e.to_string()),
        };

        // Get the absolute file path.
        let file_path = file.mangled_name().join(path);

        // If the file is a directory, create it.
        if file.is_dir() {
            match std::fs::create_dir_all(&file_path) {
                Ok(_) => (),
                Err(e) => return Some(e.to_string()),
            };
            continue;
        }

        // If the file is a file, extract it.
        let mut output_file = match std::fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => return Some(e.to_string()),
        };
        match std::io::copy(&mut file, &mut output_file) {
            Ok(_) => (),
            Err(e) => return Some(e.to_string()),
        };
    }

    // Return no error.
    None
}

// Prompt and setup PHP if possible. Returns the path.
fn setup_php(
    path: PathBuf,
    artifact: &'static ArtifactInfo,
    message: &'static str,
) -> Result<String, String> {
    // Prompt the user to download PHP.
    let confirmation = crate::mainthread::main_thread_sync(|| {
        native_dialog::MessageDialog::new()
            .set_title("PHP Required")
            .set_text(message)
            .show_confirm()
            .is_ok()
    });

    // If the user did not confirm, return an error.
    if !confirmation {
        return Err("PHP is required to run PHP based custom uploaders".to_string());
    }

    // Download the PHP binary.
    let res = match ureq::get(&artifact.url)
        .set("User-Agent", "MagicCap") // we should probably be responsible netizens downloading big things :)
        .call()
    {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };

    if artifact.zip {
        // Extract the PHP archive.
        let mut data: Vec<u8> = Vec::new();
        match res.into_reader().read_to_end(&mut data) {
            Ok(d) => d,
            Err(e) => return Err(e.to_string()),
        };
        if let Some(err) = extract_php(data, &path) {
            return Err(err);
        }
    } else {
        // Write the PHP binary.
        let mut file = match std::fs::File::create(&path) {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        };
        let mut reader = res.into_reader();
        match std::io::copy(&mut reader, &mut file) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        };
    }

    // Check the integrity of PHP.
    match check_php_integrity(&path) {
        Ok(Ok(s)) => {
            // Mark the PHP binary as executable.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = match std::fs::metadata(&s) {
                    Ok(p) => p.permissions(),
                    Err(e) => return Err(e.to_string()),
                };
                perms.set_mode(0o755);
                match std::fs::set_permissions(&s, perms) {
                    Ok(_) => (),
                    Err(e) => return Err(e.to_string()),
                };
            }

            // Return the path to the PHP binary.
            Ok(s.to_str().unwrap().to_string())
        }
        Ok(Err(_)) => Err("The downloaded PHP binary fails integrity checks".to_string()),
        Err(e) => Err(e),
    }
}

// Sets up the PHP binary and returns it if it was downloaded.
pub fn setup_php_binary(path: PathBuf) -> Result<String, String> {
    let artifact = match get_artifact_info() {
        Ok(a) => a,
        Err(e) => return Err(e),
    };
    setup_php(
        path, &artifact,
        "PHP is not found in your MagicCap installation and is required to run PHP based custom uploaders. Would you like to download it now?",
    )
}

// Return the path to the PHP CLI binary and validate the metadata.
pub fn validate_php_metadata(path: PathBuf) -> Result<String, String> {
    // Check the PHP integrity and bubble up any immediate errors to the user.
    let php_check = match check_php_integrity(&path) {
        Ok(x) => x,
        Err(e) => return Err(e),
    };

    // If the PHP binary is valid, return it.
    let artifact_info = match php_check {
        Ok(s) => return Ok(s.to_str().unwrap().to_string()),
        Err(a) => a,
    };

    // If the PHP binary is invalid, prompt the user to redownload it.
    setup_php(
        path, &artifact_info,
        "The PHP binary in your MagicCap installation is not supported with this version. Would you like to redownload it now?",
    )
}
