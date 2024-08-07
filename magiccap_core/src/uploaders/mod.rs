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
            ConfigOption::Boolean { .. } => false,
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

// Defines all of the leaked uploaders. This is honestly fine since the user won't be flooding this with custom uploaders.
pub static LEAKED_UPLOADERS: Lazy<RwLock<HashMap<String, &'static Uploader>>> = Lazy::new(|| Default::default());

// Check if the uploader is a cache hit.
fn uploader_cache_hit(hash: &str) -> Option<&'static Uploader> {
    let leaked_uploaders = LEAKED_UPLOADERS.read().unwrap();
    let res = leaked_uploaders.get(hash).copied();
    drop(leaked_uploaders);
    res
}

// Set the hash in the uploader cache.
fn uploader_cache_set(hash: String, uploader: &'static Uploader) {
    let mut leaked_uploaders = LEAKED_UPLOADERS.write().unwrap();
    leaked_uploaders.insert(hash, uploader);
    drop(leaked_uploaders);
}

// Get the uploader by ID.
pub fn get_uploader(uploader_id: &str) -> Option<&'static Uploader> {
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

    // Check if the uploader is a cache hit.
    if let Some(uploader) = uploader_cache_hit(&hash) {
        return Some(uploader);
    }

    // Deserialize the custom uploader.
    let custom_uploader: custom::CustomUploader = match serde_json::from_value(custom_uploader) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Create the uploader.
    let handler_func = custom_uploader.handler.into_uploader(uploader_id.to_string());
    let uploader = Uploader {
        name: uploader_id.to_string(),
        description: custom_uploader.description,
        icon_path: custom_uploader.encoded_icon,
        options: custom_uploader.config.into_inner(),
        upload: Box::new(handler_func),
    };

    // Leak the memory and insert into the cache.
    let uploader_mem_leak = Box::leak(Box::new(uploader));
    uploader_cache_set(hash.clone(), uploader_mem_leak);

    // Return the uploader.
    Some(uploader_mem_leak)
}

// Gets all the custom uploaders loaded into MagicCap.
pub fn get_custom_uploaders() -> HashMap<String, &'static Uploader> {
    // Get the custom uploader keys.
    let uploader_keys = match database::get_config_option("custom_uploaders") {
        Some(v) => v,
        None => return HashMap::new(),
    };
    let uploader_keys = match uploader_keys.as_array() {
        Some(v) => v,
        None => return HashMap::new(),
    };

    // Get the uploaders.
    let mut uploaders = HashMap::with_capacity(uploader_keys.len());
    for key in uploader_keys.iter() {
        // Get the key as a string.
        let key = match key.as_str() {
            Some(v) => v,
            None => continue,
        };

        // Get the uploader.
        if let Some(uploader) = get_uploader(key) {
            uploaders.insert(key.to_string(), uploader);
        }
    }
    uploaders
}

// Defines a custom uploader insert error.
pub enum CustomUploaderInsertError {
    SerializationError(String),
    AlreadyExists,
}

// Inserts a custom uploader into MagicCap.
pub fn insert_custom_uploader(uploader: custom::CustomUploader, replace: bool) -> Result<(), CustomUploaderInsertError> {
    // Serialize the uploader.
    let serialized = match serde_json::to_value(&uploader) {
        Ok(v) => v,
        Err(e) => return Err(CustomUploaderInsertError::SerializationError(e.to_string())),
    };

    // Check if the uploader already exists.
    let uploader_key = "custom_uploader_".to_string() + &uploader.name;
    if let Some(_) = database::get_config_option(&uploader_key) {
        if !replace {
            return Err(CustomUploaderInsertError::AlreadyExists);
        }
    }

    // Insert the uploader.
    database::set_config_option(&uploader_key, &serialized);
    let mut custom_uploaders = match database::get_config_option("custom_uploaders") {
        Some(v) => v,
        None => serde_json::Value::Array(Vec::new()),
    };
    let custom_uploaders = match custom_uploaders.as_array_mut() {
        Some(v) => v,
        None => return Err(CustomUploaderInsertError::SerializationError("The custom uploaders is not an array.".to_string())),
    };

    // Write the uploader key.
    custom_uploaders.push(serde_json::Value::String(uploader.name));

    // Set the custom uploaders.
    database::set_config_option("custom_uploaders", &serde_json::Value::Array(custom_uploaders.clone()));
    Ok(())
}

// Deletes a custom uploader from MagicCap.
pub fn delete_custom_uploader(uploader_name: &str) {
    // Delete the custom uploader.
    let uploader_key = "custom_uploader_".to_string() + uploader_name;
    database::delete_config_option(&uploader_key);

    // Delete the custom uploader from the custom uploaders.
    let mut custom_uploaders = match database::get_config_option("custom_uploaders") {
        Some(v) => v,
        None => serde_json::Value::Array(Vec::new()),
    };
    let custom_uploaders = match custom_uploaders.as_array_mut() {
        Some(v) => v,
        None => return,
    };
    let mut index = None;
    for (i, v) in custom_uploaders.iter().enumerate() {
        if let Some(v) = v.as_str() {
            if v == uploader_name {
                index = Some(i);
                break;
            }
        }
    }
    if let Some(i) = index {
        custom_uploaders.remove(i);
        database::set_config_option("custom_uploaders", &serde_json::Value::Array(custom_uploaders.clone()));
    }
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
