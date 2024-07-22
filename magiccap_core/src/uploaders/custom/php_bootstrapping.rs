use std::{collections::HashMap, path::PathBuf};
use once_cell::sync::Lazy;
use serde::Deserialize;

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
        None => return Err("No PHP artifact information found for the current OS/architecture".to_string()),
    };

    // Return the artifact information.
    Ok(artifact_info)
}

// Check the integrity of a PHP folder.
fn check_php_folder(path: &PathBuf, sha256: &'static str) -> Result<bool, String> {
    // TODO
    Err("Nope".to_string())
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
    let hash_str = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    // Return if they are equal.
    Ok(hash_str == sha256)
}

// Checks the integrity of the PHP path. The outer Result is for an error that we should immediately return
// to the user and not proceed with, and the inner Result is for the artifact information. If the inner Result
// is a string, that is the path to the PHP binary that should be used. If the inner Result is the artifact
// information, that means the integrity check failed and the user should be prompted to redownload the PHP binary.
fn check_php_integrity(path: &PathBuf) -> Result<Result<&str, &'static ArtifactInfo>, String> {
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
        Ok(Ok(path.to_str().unwrap()))
    } else {
        Ok(Err(artifact))
    }
}

// Prompt and setup PHP if possible. Returns the path.
fn setup_php(path: PathBuf, artifact: &'static ArtifactInfo, message: &'static str) -> Result<String, String> {
    // TODO
    Err("Nope".to_string())
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
        Ok(s) => return Ok(s.to_string()),
        Err(a) => a,
    };

    // If the PHP binary is invalid, prompt the user to redownload it.
    setup_php(
        path, &artifact_info,
        "The PHP binary in your MagicCap installation is not supported with this version. Would you like to redownload it now?",
    )
}
