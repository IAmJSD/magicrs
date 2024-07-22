pub mod custom;
mod utils;
mod elixire;
mod imgur;
mod mime;
mod shell;
mod ftp;
mod s3;
mod sftp;

use std::{collections::HashMap, sync::{atomic::Ordering, RwLock}};
use custom::IntoUploader;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use crate::{database, statics};

// Handles the config option type.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "option_type")]
pub enum ConfigOption {
    String {
        name: String,
        description: String,
        default: Option<String>,
        required: bool,
        password: bool,
        regex: Option<String>,
        validation_error_message: Option<String>,
    },
    LongString {
        name: String,
        description: String,
        default: Option<String>,
        required: bool,
    },
    Number {
        name: String,
        description: String,
        default: Option<i64>,
        min: Option<i64>,
        max: Option<i64>,
        required: bool,
    },
    Boolean {
        name: String,
        description: String,
        default: Option<bool>,
        required: bool,
    },
    Embedded {
        name: String,
        description: String,
        component_name: String,
        required: bool,
    },
    Custom {
        name: String,
        description: String,
        frame_html: String,
        required: bool,
    },
}

impl ConfigOption {
    pub fn is_required(&self) -> bool {
        match self {
            ConfigOption::String { required, .. } => *required,
            ConfigOption::LongString { required, .. } => *required,
            ConfigOption::Number { required, .. } => *required,
            ConfigOption::Boolean { required, .. } => *required,
            ConfigOption::Embedded { required, .. } => *required,
            ConfigOption::Custom { required, .. } => *required,
        }
    }
}

// Defines the type for a uploader.
#[derive(Serialize)]
pub struct Uploader {
    pub name: String,
    pub description: String,
    pub icon_path: String,
    pub options: Vec<(String, ConfigOption)>,

    #[serde(skip)]
    pub upload: Box<dyn Fn(
        &str, HashMap<String, serde_json::Value>,
        Box<dyn std::io::Read + Send + Sync>,
    ) -> Result<String, String> + Send + Sync>,
}

// Defines the uploaders.
pub static UPLOADERS: Lazy<HashMap<String, Uploader>> = Lazy::new(|| {
    let mut uploaders = HashMap::new();

    uploaders.insert("elixire".to_string(), elixire::elixire_support());
    uploaders.insert("imgur".to_string(), imgur::imgur_support());
    uploaders.insert("shell".to_string(), shell::shell_support());
    uploaders.insert("ftp".to_string(), ftp::ftp_support());
    uploaders.insert("sftp".to_string(), sftp::sftp_support());
    uploaders.insert("s3".to_string(), s3::s3_support());

    uploaders
});

// Defines all of the leaked uploaders. This is honestly fine since the user won't be flooding
// this with custom uploaders.
pub static LEAKED_UPLOADERS: Lazy<RwLock<HashMap<String, &'static Uploader>>> = Lazy::new(|| Default::default());

// Get the uploader by ID.
pub fn get_uploader(uploader_id: &str) -> Option<&Uploader> {
    // Try to get from the official uploaders.
    if let Some(uploader) = UPLOADERS.get(uploader_id) {
        return Some(uploader);
    }

    // Get the custom uploader from the configuration.
    let key = "custom_uploader_".to_string() + uploader_id;
    let custom_uploader = match database::get_config_option(&key) {
        Some(v) => v,
        None => return None,
    };

    // Hash the value.
    let h = match custom_uploader.as_object() {
        Some(v) => v,
        None => return None,
    };
    let hash = sha256::digest(
        serde_json::to_string(&h).unwrap().as_bytes()
    );

    // Check if the uploader is leaked. If so, use the leaked uploader.
    let locker = LEAKED_UPLOADERS.read().unwrap();
    if locker.contains_key(&hash) {
        return locker.get(&hash).copied();
    }

    // Parse the custom uploader.
    let custom_uploader: custom::CustomUploader = match serde_json::from_value(custom_uploader) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Return the custom uploader.
    let func = custom_uploader.handler.into_uploader(uploader_id.to_string());
    let uploader = Uploader {
        name: custom_uploader.name,
        description: custom_uploader.description,
        icon_path: custom_uploader.encoded_icon,
        options: custom_uploader.config.into_inner(),
        upload: func,
    };
    let mut locker = LEAKED_UPLOADERS.write().unwrap();
    let leak = Box::leak(Box::new(uploader));
    locker.insert(hash.clone(), leak);
    Some(leak)
}

// Calls the uploader.
pub fn call_uploader(
    uploader_name: &str, reader: Box<dyn std::io::Read + Send + Sync>, filename: &str,
) -> Result<String, String>{
    // Check if the kill switch was activated.
    if statics::KILL_SWITCH.load(Ordering::Relaxed) {
        return Err("The application is unloading.".to_string());
    }

    // Get the uploader.
    let uploader = match get_uploader(uploader_name) {
        Some(uploader) => uploader,
        None => {
            return Err(format!("The uploader {} does not exist.", uploader_name));
        },
    };

    // Validate all required options are present.
    let options = database::get_uploader_config_items(uploader_name);
    for (name, config) in &uploader.options {
        if config.is_required() && !options.contains_key(name) {
            return Err("The uploader is missing required options.".to_string());
        }
    }

    // Call the uploader.
    (uploader.upload)(filename, options, reader)
}
