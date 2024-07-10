mod elixire;
mod imgur;
mod mime;
mod shell;
mod ftp;
mod s3;

use std::{collections::HashMap, sync::atomic::Ordering};
use once_cell::sync::Lazy;
use serde::Serialize;
use crate::{database, statics};

// Handles the config option type.
#[derive(Serialize)]
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
    pub upload: fn(
        filename: &str, options: HashMap<String, serde_json::Value>,
        reader: Box<dyn std::io::Read + Send + Sync>,
    ) -> Result<String, String>,
}

// Defines the uploaders.
pub static UPLOADERS: Lazy<HashMap<String, Uploader>> = Lazy::new(|| {
    let mut uploaders = HashMap::new();

    uploaders.insert("elixire".to_string(), elixire::elixire_support());
    uploaders.insert("imgur".to_string(), imgur::imgur_support());
    uploaders.insert("shell".to_string(), shell::shell_support());
    uploaders.insert("ftp".to_string(), ftp::ftp_support());
    uploaders.insert("s3".to_string(), s3::s3_support());

    uploaders
});

// Calls the uploader.
pub fn call_uploader(
    uploader_name: &str, reader: Box<dyn std::io::Read + Send + Sync>, filename: &str,
) -> Result<String, String>{
    // Check if the kill switch was activated.
    if statics::KILL_SWITCH.load(Ordering::Relaxed) {
        return Err("The application is unloading.".to_string());
    }

    // Get the uploader.
    let uploader = match UPLOADERS.get(uploader_name) {
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
