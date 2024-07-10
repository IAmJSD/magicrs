use std::{collections::HashMap, time::SystemTime};
use super::{mime, ConfigOption, Uploader};
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, sign, SigningParams as HTTPSigningParams, SigningSettings},
    sign::v4::SigningParams,
};
use aws_credential_types::Credentials;

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

    // Build the headers.
    let acl = match config.get("acl") {
        Some(acl) => acl.as_str().unwrap(),
        None => "public-read",
    };
    let len_s = body.len().to_string();
    let mime_s = mime.to_string();
    let mut headers = vec![
        ("content-length", len_s.as_str()),
        ("content-type", mime_s.as_str()),
        ("x-amz-acl", acl),
        ("cache-control", "max-age=31536000"),
        ("accept-encoding", "gzip"),
        ("user-agent", "MagicCap"),
    ];
    // TODO: figure out why the signature is invalid

    // Perform the signing.
    let url_pre_sign = url.to_string();
    let signable_request = match SignableRequest::new(
        "PUT", url_pre_sign.as_str(), headers.iter().cloned(), SignableBody::Bytes(&body), 
    ) {
        Ok(r) => r,
        Err(e) => return Err(format!("Failed to create the signable request: {}", e)),
    };
    let identity = Credentials::new(
        config.get("access_key_id").unwrap().as_str().unwrap(),
        config.get("secret_access_key").unwrap().as_str().unwrap(),
        None, None, "hardcoded-credentials",
    ).into();
    let region = match config.get("region") {
        Some(r) => r.as_str().unwrap(),
        None => "us-east-1",
    };
    let signing_params = match SigningParams::builder().
        region(region).
        identity(&identity).
        name("s3").
        time(SystemTime::now()).
        settings(SigningSettings::default()).
        build()
    {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to create the signing params: {}", e)),
    };
    (
        "s3", region,
        config.get("access_key_id").unwrap().as_str().unwrap(),
        config.get("secret_access_key").unwrap().as_str().unwrap(),
    );
    let signing_output = match sign(
        signable_request, &HTTPSigningParams::from(signing_params),
    ) {
        Ok(r) => r,
        Err(e) => return Err(format!("Failed to sign the request: {}", e)),
    };

    // Add the headers to the request.
    let si_output = signing_output.output();
    headers.extend(si_output.headers());
    let mut query = "".to_string();
    let new_query = si_output.params();
    if !new_query.is_empty() {
        for (k, v) in new_query {
            query += k;
            query += "=";
            query += v.to_string().as_str();
            query += "&";
        }
        url.set_query(Some(query.as_str())).unwrap();
    }

    // Perform the request.
    let mut req = ureq::put(url.to_string().as_str());
    for (k, v) in &headers {
        req = req.set(k, v);
    }
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
