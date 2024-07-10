use std::collections::HashMap;
use super::{mime::guess_mime_type, ConfigOption, Uploader};
use ureq_multipart::MultipartBuilder;
use uriparse::URI;

fn elixire_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Guess the MIME type.
    let (mime, mut reader) = match guess_mime_type(filename, reader) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    // Build the multipart data from the reader.
    let multipart = MultipartBuilder::new()
        .add_stream(&mut reader, "f", Some(filename), Some(mime));

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

    // Add the required values to the URL.
    let mut url = URI::try_from("https://elixi.re/api/upload").unwrap();
    let mut query: String;
    if let Some(domain_config_json) = config.get("domain_config") {
        // Ensure it is an array if it exists and is 3 values in length.
        let arr = domain_config_json.as_array().unwrap();
        if arr.len() != 3 {
            return Err("The domain configuration must be an array of 3 values.".to_string());
        }

        // Get the subdomain (string | null), domain (string), and if it should allow subdomains (bool).
        let subdomain = match arr[0].as_str() {
            Some(s) => s,
            None => "",
        };
        let domain = arr[1].as_str().unwrap();
        let allow_subdomains = arr[2].as_bool().unwrap();

        // Add the domain to the URL params in all cases.
        query = "domain=".to_string() + &urlencoding::encode(domain);

        // If allow subdomains is set and it is not empty, add the subdomain to the URL params.
        if allow_subdomains && !subdomain.is_empty() {
            query += "&subdomain=";
            query += &urlencoding::encode(subdomain);
        }

        // Add the query to the URL.
        let s = query.as_str();
        url.set_query(Some(s)).unwrap();
    }

    // Send the request to the server.
    let resp = ureq::post(url.to_string().as_str())
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
        },
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
            ("domain_config".to_string(), ConfigOption::Embedded {
                name: "Domain Configuration".to_string(),
                description: "The configuration for the domain to use.".to_string(),
                component_name: "elixire.domain_config".to_string(),
                required: false,
            })
        ]),
        upload: elixire_support_upload,
    }
}
