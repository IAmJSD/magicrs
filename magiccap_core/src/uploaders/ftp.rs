use std::collections::HashMap;
use suppaftp::{native_tls::TlsConnector, types::FileType, NativeTlsConnector, NativeTlsFtpStream};
use super::{
    utils::{DOMAIN_OR_IP_REGEX, URL_FTP_REWRITE_DESCRIPTION}, ConfigOption, Uploader,
};

// Defines the function to upload a screenshot using FTP.
fn ftp_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    mut reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Handle the hostname.
    let mut hostname = config.get("hostname").unwrap().to_string();
    if let Some(port) = config.get("port") {
        hostname += ":";
        hostname += &port.to_string();
    }

    // Create the FTP stream.
    let use_ssl = match config.get("ssl_enabled") {
        Some(ssl_enabled) => ssl_enabled.as_bool().unwrap(),
        None => false,
    };
    let mut ftp_stream = match NativeTlsFtpStream::connect(&hostname) {
        Ok(c) => c,
        Err(err) => {
            return Err(format!("Failed to connect to the FTP server: {}", err));
        },
    };
    if use_ssl {
        let ctx = match TlsConnector::new()
        {
            Ok(tls) => tls,
            Err(err) => {
                return Err(format!("Failed to setup TLS stream: {}", err));
            }
        };
        ftp_stream = match ftp_stream.into_secure(NativeTlsConnector::from(ctx), &hostname) {
            Ok(s) => s,
            Err(err) => {
                return Err(format!("Failed to setup TLS stream: {}", err));
            }
        };
    }

    // Handle the login.
    let username = match config.get("username") {
        Some(username) => username.as_str().unwrap(),
        None => "anonymous",
    };
    let password = match config.get("password") {
        Some(password) => password.as_str().unwrap(),
        None => "anonymous",
    };
    if let Err(err) = ftp_stream.login(username, password) {
        return Err(format!("Failed to login to the FTP server: {}", err));
    }

    // Set to a binary transfer.
    if let Err(err) = ftp_stream.transfer_type(FileType::Binary) {
        return Err(format!("Failed to set the transfer type to binary: {}", err));
    }

    // If path is set, change the directory.
    let path_var: String;
    if let Some(path) = config.get("path") {
        // Add a slash to the path if it does not exist.
        let path = if path.as_str().unwrap().ends_with("/") {
            path_var = path.to_string();
            path.to_string()
        } else {
            path_var = path.to_string();
            path_var.clone() + "/"
        };

        // Change the directory.
        if let Err(err) = ftp_stream.cwd(&path) {
            return Err(format!("Failed to change the directory to {}: {}", path, err));
        }
    } else {
        path_var = "".to_string();
    }

    // Put the file.
    if let Err(err) = ftp_stream.put_file(filename, &mut reader) {
        return Err(format!("Failed to upload the file to the FTP server: {}", err));
    }

    // Close the connection.
    if let Err(err) = ftp_stream.quit() {
        return Err(format!("Failed to close the connection to the FTP server: {}", err));
    }

    // Process the URL rewrite.
    let url_rewrite = match config.get("url_rewrite") {
        Some(url_rewrite) => url_rewrite.as_str().unwrap(),
        None => "https://$hostname$folder_path/$filename",
    };
    let hostname_pre_port = hostname.split(":").next().unwrap();
    let url = url_rewrite
        .replace("$hostname", hostname_pre_port)
        .replace("$folder_path", &path_var)
        .replace("$filename", &urlencoding::encode(filename));

    // Return the URL.
    Ok(url)
}

// Defines the config structure for FTP.
pub fn ftp_support() -> Uploader {
    Uploader {
        name: "FTP".to_string(),
        description: "Uploads the screenshot using FTP.".to_string(),
        icon_path: "/icons/ftp.png".to_string(),
        options: vec![
            (
                "hostname".to_string(),
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
                "port".to_string(),
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
                "ssl_enabled".to_string(),
                ConfigOption::Boolean {
                    name: "SSL Enabled".to_string(),
                    description: "Whether to use SSL for the connection.".to_string(),
                    default: None,
                    required: false,
                },
            ),
            (
                "username".to_string(),
                ConfigOption::String {
                    name: "Username".to_string(),
                    description: "The username to use for the FTP server.".to_string(),
                    default: None,
                    required: false,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "password".to_string(),
                ConfigOption::String {
                    name: "Password".to_string(),
                    description: "The password to use for the FTP server.".to_string(),
                    default: None,
                    required: false,
                    password: true,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "path".to_string(),
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
                "url_rewrite".to_string(),
                ConfigOption::String {
                    name: "URL Rewrite".to_string(),
                    description: URL_FTP_REWRITE_DESCRIPTION.to_string(),
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
