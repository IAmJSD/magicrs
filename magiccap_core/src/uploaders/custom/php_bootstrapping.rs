use std::{collections::HashMap, fs::Metadata, path::PathBuf};
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

// Sets up the PHP binary and returns it if it was downloaded.
pub fn setup_php_binary(path: PathBuf) -> Result<String, String> {
    // TODO
    Err("Not implemented".to_string())
}

// Return the path to the PHP CLI binary and validate the metadata.
pub fn validate_php_metadata(path: PathBuf, metadata: Metadata) -> Result<String, String> {
    // TODO
    Err("Not implemented".to_string())
}
