use std::collections::HashMap;
use super::{ConfigOption, Uploader};

fn ftp_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    return Err("fucky wucky uwu".to_string())
}

// Defines a regex for RFC 1123 compliant domain names or IP addresses.
const DOMAIN_OR_IP_REGEX: &str = "^(([a-zA-Z]|[a-zA-Z][a-zA-Z0-9\\-]*[a-zA-Z0-9])\\.)*([A-Za-z]|[A-Za-z][A-Za-z0-9\\-]*[A-Za-z0-9])$";

// Defines the description for URL rewrites since it is fairly long.
const URL_REWRITE_DESCRIPTION: &str = concat!(
    "The string to rewrite the URL to. In this URL, you can use `$hostname` to represent the hostname, ",
    "`$folder_path` to represent the folder path, and `$filename` to represent the filename. The default ",
    "is `https://$hostname$folder_path/$filename`.",
);

pub fn ftp_support() -> Uploader {
    Uploader {
        name: "FTP".to_string(),
        description: "Uploads the screenshot using FTP.".to_string(),
        icon_path: "/icons/ftp.svg".to_string(),
        options: vec![
            (
                "Hostname".to_string(),
                ConfigOption::String {
                    name: "Hostname".to_string(),
                    description: "The hostname of the FTP server.".to_string(),
                    default: None,
                    required: true,
                    password: false,
                    regex: Some(DOMAIN_OR_IP_REGEX.to_string()),
                    validation_error_message: Some("The hostname is not a valid domain or IP address.".to_string()),
                },
            ),
            (
                "Port".to_string(),
                ConfigOption::Number {
                    name: "Port".to_string(),
                    description: "The port of the FTP server. Defaults to 21.".to_string(),
                    default: Some(21),
                    required: false,
                    min: Some(1),
                    max: Some(65535),
                },
            ),
            (
                "SSL Enabled".to_string(),
                ConfigOption::Boolean {
                    name: "SSL Enabled".to_string(),
                    description: "Whether to use SSL for the connection.".to_string(),
                    default: None,
                    required: false,
                },
            ),
            (
                "Username".to_string(),
                ConfigOption::String {
                    name: "Username".to_string(),
                    description: "The username to use for the FTP server.".to_string(),
                    default: None,
                    required: true,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "Password".to_string(),
                ConfigOption::String {
                    name: "Password".to_string(),
                    description: "The password to use for the FTP server.".to_string(),
                    default: None,
                    required: true,
                    password: true,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "Path".to_string(),
                ConfigOption::String {
                    name: "Folder Path".to_string(),
                    description: "The folder path to upload the screenshot to within the FTP server.".to_string(),
                    default: None,
                    required: false,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "URL Rewrite".to_string(),
                ConfigOption::String {
                    name: "URL Rewrite".to_string(),
                    description: URL_REWRITE_DESCRIPTION.to_string(),
                    default: None,
                    required: false,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
        ],
        upload: ftp_support_upload,
    }
}
