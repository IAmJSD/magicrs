use std::collections::HashMap;
use super::{mime::guess_mime_type, ConfigOption, Uploader};
use ureq_multipart::MultipartBuilder;

fn elixire_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Guess the MIME type.
    let (mime, reader) = match guess_mime_type(filename, reader) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    // Build the multipart data from the reader.
    let mut r = reader;
    let multipart = MultipartBuilder::new()
        .add_stream(&mut r, "f", Some(filename), Some(mime));

    // Check that the stream was consumed okay.
    let multipart = match multipart {
        Ok(m) => m,
        Err(err) => return Err(err.to_string()),
    };

    // Finish the multipart data.
    let (content_type, multipart) = match multipart.finish() {
        Ok(m) => m,
        Err(err) => return Err(err.to_string()),
    };

    // Send the request to the server.
    let resp = ureq::post("https://elixi.re/api/upload")
        .set("Authorization", &config["token"].as_str().unwrap())
        .set("Content-Type", &content_type)
        .send_bytes(&multipart);

    // Handle the result.
    match resp {
        Ok(resp) => {
            let json: serde_json::Value = match resp.into_json() {
                Ok(json) => json,
                Err(err) => return Err(err.to_string()),
            };
            match json["url"].as_str() {
                Some(link) => Ok(link.to_string()),
                None => Err("The response did not contain a link.".to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub fn elixire_support() -> Uploader {
    Uploader {
        name: "elixi.re".to_string(),
        description: "elixire is the future".to_string(),
        icon_path: "/icons/elixire.svg".to_string(),
        options: HashMap::from([
            ("token".to_string(), ConfigOption::String {
                name: "Token".to_string(),
                description: "The token to use for the elixi.re API.".to_string(),
                default: None,
                required: true,
                password: true,
                regex: None,
                validation_error_message: None,
            }),
        ]),
        upload: elixire_support_upload,
    }
}
