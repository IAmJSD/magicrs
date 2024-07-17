use std::collections::HashMap;
#[allow(unused_imports)]
use std::process::{Command, Child};
use super::{
    utils::{DOMAIN_OR_IP_REGEX, URL_FTP_REWRITE_DESCRIPTION},
    ConfigOption, Uploader,
};

// Defines an atomic that increments to make sure we do not collide with other threads.
#[cfg(target_os = "windows")]
static SFTP_UPLOAD_COUNTER: once_cell::sync::Lazy<std::sync::atomic::AtomicUsize> =
    once_cell::sync::Lazy::new(|| std::sync::atomic::AtomicUsize::new(0));

// Handles getting an agent socket and child on Windows.
#[cfg(target_os = "windows")]
fn agent_socket() -> Result<(String, Child), String> {
    // Get a random file path for the socket.
    let sock_filename = format!(
        "magiccap-ssh-agent-{}.sock",
        SFTP_UPLOAD_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
    );
    let tmp_socket = std::env::temp_dir().join(&sock_filename).to_str().unwrap().to_string();

    // Call pageant to get a socket.
    let child = match Command::new("pageant")
        .arg("--unix")
        .arg(&tmp_socket)
        .spawn()
    {
        Ok(child) => child,
        Err(err) => return Err(err.to_string()),
    };

    // Return the socket and child process.
    Ok((tmp_socket.to_string(), child))
}

// Handles getting an agent socket and PID on macOS/Linux.
#[cfg(not(target_os = "windows"))]
fn agent_socket() -> Result<(String, usize), String> {
    // Call ssh-agent to get a socket and PID.
    let output = match Command::new("ssh-agent").output() {
        Ok(output) => output,
        Err(err) => return Err(err.to_string()),
    };
    let output = String::from_utf8(output.stdout).unwrap();

    // Get the socket and PID.
    let socket = output
        .lines()
        .find(|line| line.starts_with("SSH_AUTH_SOCK="))
        .unwrap()
        .split('=')
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let pid = output
        .lines()
        .find(|line| line.starts_with("SSH_AGENT_PID="))
        .unwrap()
        .split('=')
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .parse::<usize>()
        .unwrap();

    // Return the socket and child process.
    Ok((socket.to_string(), pid))
}

struct SSHConnectionOptions {
    agent_socket: String,
    hostname: String,
    port: u16,
    username: String,
    private_key: Option<String>,
    password: Option<String>,
}

fn sftp_do_upload(
    connection: SSHConnectionOptions, folder_path: &Option<String>,
    filename: &str, reader: Box<dyn std::io::Read + Send + Sync>,
) -> Option<String> {
    // TODO
    Some("oops".to_string())
}

fn sftp_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Get the hostname.
    let hostname = match config.get("hostname") {
        Some(hostname) => match hostname.as_str() {
            Some(hostname) => hostname,
            None => return Err("The hostname must be a string.".to_string()),
        },
        None => return Err("The hostname is required.".to_string()),
    };

    // Get the port.
    let port = match config.get("port") {
        Some(port) => match port.as_u64() {
            Some(port) => port as u16,
            None => return Err("The port must be a number.".to_string()),
        },
        None => 22,
    };

    // Get the username.
    let username = match config.get("username") {
        Some(username) => match username.as_str() {
            Some(username) => username.to_string(),
            None => return Err("The username must be a string.".to_string()),
        },
        None => whoami::username(),
    };

    // Get the private key and password.
    let private_key = match config.get("private_key") {
        Some(private_key) => match private_key.as_str() {
            Some(private_key) => Some(private_key.to_string()),
            None => return Err("The private key must be a string.".to_string()),
        },
        None => None,
    };
    let password = match config.get("password") {
        Some(password) => match password.as_str() {
            Some(password) => Some(password.to_string()),
            None => return Err("The password must be a string.".to_string()),
        },
        None => None,
    };

    // Call the command handling function.
    let folder_path = match config.get("path") {
        Some(folder_path) => match folder_path.as_str() {
            Some(folder_path) => Some(folder_path.to_string()),
            None => return Err("The folder path must be a string.".to_string()),
        },
        None => None,
    };

    // Start the SSH agent.
    #[allow(unused_mut)] // This is only used on Windows.
    let (socket, mut pid_or_child) = match agent_socket() {
        Ok((socket, pid_or_child)) => (socket, pid_or_child),
        Err(err) => return Err(err),
    };

    // Call the handler function and then kill the agent whatever happens.
    let possible_error = sftp_do_upload(
        SSHConnectionOptions {
            agent_socket: socket,
            hostname: hostname.to_string(),
            port,
            username,
            private_key,
            password,
        },
        &folder_path,
        filename,
        reader,
    );
    #[cfg(target_os = "windows")]
    {
        let _ = pid_or_child.kill();
        drop(pid_or_child);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = unsafe { libc::kill(pid_or_child as i32, libc::SIGKILL) };
    }
    if let Some(error) = possible_error {
        return Err(error);
    }

    // Remove the slash from the end of the folder path if it exists.
    let folder_path = match folder_path {
        Some(folder_path) => {
            if folder_path.ends_with('/') {
                folder_path[..folder_path.len() - 1].to_string()
            } else {
                folder_path
            }
        },
        None => "".to_string(),
    };

    // Get the URL rewrite.
    let url_rewrite = match config.get("url_rewrite") {
        Some(url_rewrite) => match url_rewrite.as_str() {
            Some(url_rewrite) => url_rewrite.to_string(),
            None => return Err("The URL rewrite must be a string.".to_string()),
        },
        None => "https://$hostname$folder_path/$filename".to_string(),
    };
    Ok(
        url_rewrite
            .replace("$hostname", &hostname)
            .replace("$folder_path", &folder_path)
            .replace("$filename", filename),
    )
}

const AGENT_NAME: &str = if cfg!(target_os = "windows") {
    "pageant (PuTTY)"
} else {
    "ssh-agent"
};

pub fn sftp_support() -> Uploader {
    Uploader {
        name: "SFTP".to_string(),
        description: "Uploads the screenshot using SFTP.".to_string(),
        icon_path: "/icons/sftp.png".to_string(),
        options: vec![
            (
                "hostname".to_string(),
                ConfigOption::String {
                    name: "Hostname".to_string(),
                    description: "The hostname of the SSH server.".to_string(),
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
                    description: "The port of the SSH server. Defaults to 22.".to_string(),
                    default: Some(22),
                    required: false,
                    min: Some(1),
                    max: Some(65535),
                },
            ),
            (
                "username".to_string(),
                ConfigOption::String {
                    name: "Username".to_string(),
                    description: "The username to use for the SSH server. Defaults to the OS username.".to_string(),
                    default: None,
                    required: false,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "private_key".to_string(),
                ConfigOption::LongString {
                    name: "Private Key".to_string(),
                    description: format!(
                        "The private key to use for the SSH server. If this and password is unset, uses {} as the agent.",
                        AGENT_NAME,
                    ),
                    default: None,
                    required: false,
                },
            ),
            (
                "password".to_string(),
                ConfigOption::String {
                    name: "Password".to_string(),
                    description: "The password to use for the SSH server. Prefers private key authentication.".to_string(),
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
                    description: "The folder path to upload the screenshot to within the SSH server.".to_string(),
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
        upload: sftp_support_upload,
    }
}
