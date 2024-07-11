use std::collections::HashMap;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{primitives::ByteStream, types::ObjectCannedAcl, Client};
use aws_credential_types::Credentials;

use super::{mime, ConfigOption, Uploader};

async fn s3_async_task(
    filename: &str, config: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    // Get the MIME type of the file.
    let (mime, mut reader) = match mime::guess_mime_type(filename, reader) {
        Ok((mime, reader)) => (mime, reader),
        Err(err) => return Err(err.to_string()),
    };

    // Get the endpoint, access key ID, and secret access key.
    let endpoint = match config.get("endpoint") {
        Some(endpoint) => match endpoint.as_str() {
            Some(endpoint) => endpoint,
            None => return Err("The endpoint is not a string.".to_string()),
        },
        None => return Err("The endpoint is missing.".to_string()),
    };
    let access_key_id = match config.get("access_key_id") {
        Some(access_key_id) => match access_key_id.as_str() {
            Some(access_key_id) => access_key_id,
            None => return Err("The access key ID is not a string.".to_string()),
        },
        None => return Err("The access key ID is missing.".to_string()),
    };
    let secret_access_key = match config.get("secret_access_key") {
        Some(secret_access_key) => match secret_access_key.as_str() {
            Some(secret_access_key) => secret_access_key,
            None => return Err("The secret access key is not a string.".to_string()),
        },
        None => return Err("The secret access key is missing.".to_string()),
    };

    // Get the bucket.
    let bucket = match config.get("bucket") {
        Some(bucket) => match bucket.as_str() {
            Some(bucket) => bucket,
            None => return Err("The bucket is not a string.".to_string()),
        },
        None => return Err("The bucket is missing.".to_string()),
    };

    // Get the region.
    let mut region = match config.get("region") {
        Some(region) => match region.as_str() {
            Some(region) => region,
            None => return Err("The region is not a string.".to_string()),
        },
        None => "us-east-1",
    };
    if region.is_empty() {
        region = "us-east-1";
    }

    // Leak the region to avoid the lifetime issue. This doesn't really matter because it is tiny.
    let region = Box::leak(Box::new(region.to_string()));

    // Read the file into a buffer.
    let mut data = Vec::new();
    reader.read_to_end(&mut data).map_err(|err| format!("Failed to read the file: {}.", err))?;

    // Setup the S3 client.
    let region_provider = RegionProviderChain::first_try(region.as_str()).or_default_provider();
    let sdk_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .credentials_provider(Credentials::new(
            access_key_id.to_string(),
            secret_access_key.to_string(),
            None, None, "magiccap",
        ))
        .endpoint_url("https://".to_string() + endpoint)
        .load()
        .await;
    let client = Client::new(&sdk_config);

    // Run PutObject.
    let mut folder = match config.get("folder") {
        Some(folder) => match folder.as_str() {
            Some(folder) => folder,
            None => return Err("The folder is not a string.".to_string()),
        },
        None => "",
    };
    folder = folder.trim_matches('/');
    let acl = match config.get("acl") {
        Some(acl) => match acl.as_str() {
            Some(acl) => acl,
            None => return Err("The ACL is not a string.".to_string()),
        },
        None => "public-read",
    };
    let folder_plus_slash = if folder.is_empty() {
        "".to_string()
    } else {
        format!("{}/", folder)
    };
    client.put_object()
        .bucket(bucket)
        .key(format!("{}{}", folder_plus_slash, filename))
        .acl(ObjectCannedAcl::from(acl))
        .body(ByteStream::from(data))
        .content_type(mime.to_string())
        .send()
        .await
        .map_err(|err| format!("Failed to upload the file: {}.", err))?;

    // Get the URL rewrite.
    let url_rewrite = match config.get("url_rewrite") {
        Some(url_rewrite) => match url_rewrite.as_str() {
            Some(url_rewrite) => url_rewrite,
            None => return Err("The URL rewrite is not a string.".to_string()),
        },
        None => "https://$bucket/$folder_path$filename",
    };

    // Return the URL.
    Ok(url_rewrite
        .replace("$bucket", bucket)
        .replace("$folder_path", folder)
        .replace("$filename", filename))
}

fn s3_support_upload(
    filename: &str, config: HashMap<String, serde_json::Value>,
    reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<String, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(s3_async_task(filename, config, reader))
}

// Make sure this is a valid domain.
const DOMAIN_REGEX: &str = r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$";

// Defines the description for URL rewrites since it is fairly long.
const URL_REWRITE_DESCRIPTION: &str = concat!(
    "The string to rewrite the URL to. In this URL, you can use `$bucket` to represent the bucket, ",
    "`$folder_path` to represent the folder path, and `$filename` to represent the filename. The default ",
    "is `https://$bucket/$folder_path$filename`.",
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
