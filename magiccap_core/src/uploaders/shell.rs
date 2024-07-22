use std::{collections::HashMap, process::{Command, Stdio}};
use super::{ConfigOption, Uploader};

fn shell_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    mut reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Get the command from the config.
    let command = match config.get("command") {
        Some(c) => match c.as_str() {
            Some(c) => c,
            None => return Err("The command must be a string.".to_string()),
        },
        None => return Err("The command is required.".to_string()),
    };

    // Get the users preferred shell.
    let shell = match std::env::var("SHELL") {
        Ok(shell) => shell,
        Err(_) => "/bin/sh".to_string(),
    };

    // Run the command.
    let process = Command::new(shell)
        .arg("-c")
        .arg(command)
        .env("FILENAME", filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    // Unwrap any errors starting the process.
    let mut process = match process {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to start the process: {}", e)),
    };

    // Handle writing the reader to the stdin.
    let mut stdin = process.stdin.take().unwrap();
    if let Some(_) = std::io::copy(&mut reader, &mut stdin).err() {
        // Make a best effort to kill the process.
        let _ = process.kill();

        // Return the error.
        return Err("Failed to write the reader to the stdin.".to_string());
    }

    // Wait for the process to finish and output the result.
    match process.wait_with_output() {
        Ok(output) => {
            if !output.status.success() {
                return Err(format!("The command failed with a non-zero exit code: {}", output.status));
            }
            match String::from_utf8(output.stdout) {
                Ok(url) => Ok(url),
                Err(e) => return Err(format!("Failed to read the URL from the stdout: {}", e)),
            }
        },
        Err(e) => {
            Err(format!("Failed to wait for the process to finish: {}", e))
        },
    }
}

const DESCRIPTION: &str = concat!(
    "Uploads the screenshot using a shell program. The filename is passed in with the FILENAME environment variable. ",
    "The shell script should return a 0 on success and put the URL in the stdout."
);

pub fn shell_support() -> Uploader {
    Uploader {
        name: "Shell".to_string(),
        description: DESCRIPTION.to_string(),
        icon_path: "/icons/shell.png".to_string(),
        options: vec![
            (
                "command".to_string(),
                ConfigOption::String {
                    name: "Command".to_string(),
                    description: "The command to run to upload the screenshot.".to_string(),
                    default: None,
                    required: true,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
        ],
        upload: Box::new(shell_support_upload),
    }
}
