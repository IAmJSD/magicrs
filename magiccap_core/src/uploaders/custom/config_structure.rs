use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Handles removing Embedded items from the custom uploader config.
// This is because these are meant for internal use only.
pub struct CustomUploaderConfig(Vec<(String, crate::uploaders::ConfigOption)>);

impl Serialize for CustomUploaderConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as an array of tuples but make sure none are Embedded.
        let v: Vec<_> = self.0.iter().filter_map(|(key, value)| {
            if let crate::uploaders::ConfigOption::Embedded { .. } = value {
                None
            } else {
                Some((key, value))
            }
        }).collect();
        v.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CustomUploaderConfig {
    fn deserialize<D>(deserializer: D) -> Result<CustomUploaderConfig, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Call the deserializer on the inner type.
        let v: Vec<(String, crate::uploaders::ConfigOption)> = Deserialize::deserialize(deserializer)?;

        // Make sure there are no Embedded items.
        for (_, value) in &v {
            if let crate::uploaders::ConfigOption::Embedded { .. } = value {
                return Err(serde::de::Error::custom("Embedded items are not allowed in custom uploader config"));
            }
        }

        // Return the custom uploader config.
        Ok(CustomUploaderConfig(v))
    }
}

impl CustomUploaderConfig {
    pub fn into_inner(self) -> Vec<(String, crate::uploaders::ConfigOption)> {
        self.0
    }
}

// Defines the IntoUploader trait.
pub trait IntoUploader {
    fn into_uploader(self, id: String) -> Box<dyn Fn(
        &str, HashMap<String, serde_json::Value>,
        Box<dyn std::io::Read + Send + Sync>,
    ) -> Result<String, String> + Send + Sync>;
}

// Defines the HTTP method type.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
}

// Turns the method into a str.
impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
        }
    }
}

// Defines the HTTP uploader config.
#[derive(Deserialize, Serialize)]
pub struct HTTPUploaderConfig {
    pub rewrites: HashMap<String, super::HTTPRewrite>,
    pub url_template: String,
    pub method: Method,
    pub header_templates: HashMap<String, String>,
    pub body: super::HTTPBody,
    pub response_expr: String,
}

// Implements the IntoUploader trait for HTTPUploaderConfig.
impl IntoUploader for HTTPUploaderConfig {
    fn into_uploader(self, _: String) -> Box<dyn Fn(
        &str, HashMap<String, serde_json::Value>,
        Box<dyn std::io::Read + Send + Sync>,
    ) -> Result<String, String> + Send + Sync> {
        return Box::new(move |filename, config, reader| {
            let (mime_type, reader) = match crate::uploaders::mime::guess_mime_type(filename, reader) {
                Ok(v) => v,
                Err(e) => return Err(e.to_string()),
            };
            super::http::http(
                filename, mime_type.essence_str(),
                self.rewrites.clone(), &self.url_template, self.method.as_str(),
                self.header_templates.clone(), self.body.clone(), config.clone(),
                reader, &self.response_expr,
            )
        })
    }
}

// Defines the PHP uploader config.
#[derive(Deserialize, Serialize)]
pub struct PHPUploaderConfig {
    pub code: String,
}

// Implements the IntoUploader trait for PHPUploaderConfig.
impl IntoUploader for PHPUploaderConfig {
    fn into_uploader(self, id: String) -> Box<dyn Fn(
        &str, HashMap<String, serde_json::Value>,
        Box<dyn std::io::Read + Send + Sync>,
    ) -> Result<String, String> + Send + Sync> {
        return Box::new(move |filename, config, reader| {
            super::php::php(
                &id, &self.code, config,
                filename, reader,
            )
        })
    }
}

// Defines the enum for the handler types.
#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomUploaderHandler {
    HTTP(HTTPUploaderConfig),
    PHP(PHPUploaderConfig),
}

// Implements the IntoUploader trait for CustomUploaderHandler.
impl IntoUploader for CustomUploaderHandler {
    fn into_uploader(self, id: String) -> Box<dyn Fn(
        &str, HashMap<String, serde_json::Value>,
        Box<dyn std::io::Read + Send + Sync>,
    ) -> Result<String, String> + Send + Sync> {
        match self {
            CustomUploaderHandler::HTTP(v) => v.into_uploader(id),
            CustomUploaderHandler::PHP(v) => v.into_uploader(id),
        }
    }
}

// Currently the only version is 1.
#[derive(Deserialize, Serialize)]
pub enum UploaderVersions {
    V1 = 1,
}

// Defines the main custom uploader struct.
#[derive(Deserialize, Serialize)]
pub struct CustomUploader {
    pub version: UploaderVersions,
    pub id: String,
    pub name: String,
    pub description: String,
    pub encoded_icon: String,
    pub config: CustomUploaderConfig,
    pub handler: CustomUploaderHandler,
}
