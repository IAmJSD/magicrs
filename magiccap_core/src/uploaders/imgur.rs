use std::collections::HashMap;
use super::Uploader;

fn imgur_support_upload(
    _: &str, _: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    let resp = ureq::post("https://api.imgur.com/3/image")
        .set("Authorization", "Client-ID 7be846b9a91ad50")
        .send(reader);

    match resp {
        Ok(resp) => {
            let json: serde_json::Value = match resp.into_json() {
                Ok(json) => json,
                Err(err) => return Err(err.to_string()),
            };
            match json["data"]["link"].as_str() {
                Some(link) => Ok(link.to_string()),
                None => Err("The response did not contain a link.".to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub fn imgur_support() -> Uploader {
    Uploader {
        name: "imgur".to_string(),
        description: "Uploads the image to imgur.".to_string(),
        icon_path: "/icons/imgur.svg".to_string(),
        options: HashMap::new(),
        upload: imgur_support_upload,
    }
}
