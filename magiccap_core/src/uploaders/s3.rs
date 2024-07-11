use std::collections::HashMap;
use hmac::{Hmac, Mac};
use ureq::Middleware;
use super::{mime, ConfigOption, Uploader};

struct S3Signing {
    access_key_id: String,
    secret_access_key: String,
    body_hash: String,
    region: String,
}

fn s3_url_encode(content: &str) -> String {
    let mut encoded = String::new();
    for c in content.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '~' | '.' | '/' => {
                encoded.push(c);
            },
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", c as u8));
            },
        }
    }
    encoded
}

const SIGNED_HEADERS: [&str; 4] = [
    "content-type", "x-amz-acl", "x-amz-date", "x-amz-expires",
];

impl Middleware for S3Signing {
    fn handle(&self, mut request: ureq::Request, next: ureq::MiddlewareNext) -> Result<ureq::Response, ureq::Error> {
        // Add the X-Amz-Date.
        let iso_time = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        request = request.set("X-Amz-Date", &iso_time);

        // Add the start method/URI.
        let url = request.request_url().unwrap();
        let canonical_uri = url.path();
        let mut string_pointers = Vec::new();
        string_pointers.push(request.method());
        let path_s3e = s3_url_encode(canonical_uri);
        string_pointers.push(&path_s3e);

        // Add the query parameters.
        let mut q = url.query_pairs();
        q.sort_by(|a, b| a.0.cmp(&b.0));
        let query = q.into_iter().map(|(k, v)| {
            s3_url_encode(k) + "=" + &s3_url_encode(v)
        }).collect::<Vec<_>>().join("&");
        if !query.is_empty() {
            string_pointers.push(&query);
        }

        // Add the headers.
        let mut header_names = request.header_names().into_iter().map(|h| {
            h.to_lowercase()
        }).filter(|a| {
            SIGNED_HEADERS.contains(&a.as_str())
        }).collect::<Vec<_>>();
        header_names.push("host".to_string());
        header_names.sort();
        let mut headers = "".to_string();
        for header_name in &header_names {
            let header = if header_name == "host" {
                "host:".to_string() + url.host()
            } else {
                let header_value = request.header(&header_name).unwrap();
                header_name.to_string() + ":" + header_value
            };
            if !headers.is_empty() {
                headers.push('\n');
            }
            headers.push_str(&header);
        }
        let signed_headers = header_names.join(";");
        string_pointers.push(&signed_headers);

        // Add the body to the string pointers.
        string_pointers.push(&self.body_hash);

        // Create the canonical request.
        let everything_nl_joined = string_pointers.join("\n");
        let canonical_request = sha256::digest(everything_nl_joined.as_bytes());

        // Create the string to sign.
        let string_to_sign = "AWS4-HMAC-SHA256\n".to_string() +
            &iso_time + "\n" +
            &iso_time[0..8] + &format!("/{}/s3/aws4_request\n", s3_url_encode(&self.region)) +
            &canonical_request;

        // Create the signing key.
        let hmac_sha256 = |a, b| {
            let mut mac = Hmac::<sha2::Sha256>::new_from_slice(a).unwrap();
            mac.update(b);
            mac.finalize().into_bytes()
        };
        let aws4_headed_key = format!("AWS4{}", self.secret_access_key);
        let date_key = hmac_sha256(
            aws4_headed_key.as_bytes(),
            iso_time[0..8].as_bytes(),
        ).to_vec();
        let date_region_key = hmac_sha256(&date_key, &self.region.as_bytes());
        let date_region_service_key = hmac_sha256(
            &date_region_key, "s3".as_bytes());
        let signing_key = hmac_sha256(
            &date_region_service_key, "aws4_request".as_bytes());

        // Use the signature to create the authorization header.
        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={},SignedHeaders={},Signature={:x}",
            format!("{}/{}/{}/s3/aws4_request", self.access_key_id, &iso_time[0..8], self.region),
            signed_headers,
            hmac_sha256(&signing_key, string_to_sign.as_bytes()),
        );
        request = request.set("Authorization", &auth_header);

        // Log out all the headers.
        let header_names = request.header_names();
        for header_name in header_names {
            let header_value = request.header(&header_name).unwrap();
            println!("{}: {}", header_name, header_value);
        }

        // Continue the request.
        next.handle(request)
    }
}

fn s3_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Get the mime type of the file.
    let (mime, mut reader) = match mime::guess_mime_type(filename, reader) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    // Get the folder path.
    let folder = match config.get("folder") {
        Some(folder) => {
            let v = folder.as_str().unwrap();
            let v = if v.starts_with('/') {
                // Remove the leading slash.
                &v[1..]
            } else {
                v
            };
            if v.ends_with('/') {
                // Remove the trailing slash.
                &v[..v.len() - 1]
            } else {
                v
            }
        },
        None => "",
    };

    // Get the bucket.
    let bucket = config.get("bucket").unwrap().as_str().unwrap();

    // Get the endpoint of the S3 bucket.
    let mut endpoint = config.get("endpoint").unwrap().as_str().unwrap();
    if endpoint.starts_with('.') {
        // Remove the leading period.
        endpoint = &endpoint[1..];
    }
    let endpoint = "https://".to_string() + bucket + "." + endpoint;
    let mut url = uriparse::URI::try_from(endpoint.as_str()).unwrap();

    // Get the URL path.
    let slash_or_not = if folder.is_empty() { "" } else { "/" };
    let filename = urlencoding::encode(filename);
    let url_path = folder.to_string() + slash_or_not + &filename;
    url.set_path(url_path.as_str()).unwrap();

    // Read out the entire body.
    let mut body = Vec::new();
    if let Err(e) = reader.read_to_end(&mut body) {
        return Err(format!("Failed to read the body: {}", e));
    }

    let mut region = config.get("region").unwrap_or(
        &serde_json::Value::String("us-east-1".to_string())).as_str().unwrap().to_string();
    if region == "" {
        region = "us-east-1".to_string();
    }
    let agent = ureq::builder().middleware(S3Signing {
        access_key_id: config.get("access_key_id").unwrap().as_str().unwrap().to_string(),
        secret_access_key: config.get("secret_access_key").unwrap().as_str().unwrap().to_string(),
        body_hash: sha256::digest(&body),
        region,
    }).build();
    let req = agent.put(url.to_string().as_str()).
        set("Content-Type", &mime.to_string()).
        set("x-amz-acl", config.get("acl").unwrap_or(
            &serde_json::Value::String("public-read".to_string())).as_str().unwrap()).
        set("User-Agent", "MagicCap/3 (magiccap.org)");

    // Perform the request.
    if let Err(e) = req.send_bytes(&body) {
        match e {
            ureq::Error::Status(code, response) => {
                return Err(format!("Failed to upload the file to the S3 bucket: {}: {}", code, response.into_string().unwrap()));
            },
            _ => {
                return Err(format!("Failed to upload the file to the S3 bucket: {}", e));
            },
        }
    }

    // Get the URL rewrite.
    let url_rewrite = match config.get("url_rewrite") {
        Some(url_rewrite) => url_rewrite.as_str().unwrap(),
        None => "https://$bucket/$folder_path/$filename",
    };

    // Perform the URL rewrite.
    Ok(
        url_rewrite.replace("$bucket", bucket)
            .replace("$folder_path", folder)
            .replace("$filename", &filename),
    )
}

// Make sure this is a valid domain.
const DOMAIN_REGEX: &str = r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$";

// Defines the description for URL rewrites since it is fairly long.
const URL_REWRITE_DESCRIPTION: &str = concat!(
    "The string to rewrite the URL to. In this URL, you can use `$bucket` to represent the bucket, ",
    "`$folder_path` to represent the folder path, and `$filename` to represent the filename. The default ",
    "is `https://$bucket/$folder_path/$filename`.",
);

pub fn s3_support() -> Uploader {
    Uploader {
        name: "S3".to_string(),
        description: "Uploads the screenshot to an S3 bucket.".to_string(),
        icon_path: "/icons/s3.png".to_string(),
        options: vec![
            (
                "endpoint".to_string(),
                ConfigOption::String {
                    name: "Endpoint".to_string(),
                    description: "The endpoint of the S3 bucket.".to_string(),
                    default: None,
                    required: true,
                    password: false,
                    regex: Some(DOMAIN_REGEX.to_string()),
                    validation_error_message: Some("The endpoint is not a valid domain.".to_string()),
                },
            ),
            (
                "access_key_id".to_string(),
                ConfigOption::String {
                    name: "Access Key ID".to_string(),
                    description: "The access key ID for the S3 bucket.".to_string(),
                    default: None,
                    required: true,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "secret_access_key".to_string(),
                ConfigOption::String {
                    name: "Secret Access Key".to_string(),
                    description: "The secret access key for the S3 bucket.".to_string(),
                    default: None,
                    required: true,
                    password: true,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "bucket".to_string(),
                ConfigOption::String {
                    name: "Bucket".to_string(),
                    description: "The bucket to upload the file to.".to_string(),
                    default: None,
                    required: true,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "url_rewrite".to_string(),
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
            (
                "region".to_string(),
                ConfigOption::String {
                    name: "Region".to_string(),
                    description: "The region of the S3 bucket. Defaults to `us-east-1`.".to_string(),
                    default: None,
                    required: false,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "folder".to_string(),
                ConfigOption::String {
                    name: "Folder".to_string(),
                    description: "The folder to upload the file to.".to_string(),
                    default: None,
                    required: false,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
            (
                "acl".to_string(),
                ConfigOption::String {
                    name: "ACL".to_string(),
                    description: "The ACL for the file. Defaults to `public-read`.".to_string(),
                    default: Some("public-read".to_string()),
                    required: false,
                    password: false,
                    regex: None,
                    validation_error_message: None,
                },
            ),
        ],
        upload: s3_support_upload,
    }
}
