// Exports all the logic for HTTP based custom uploaders.
mod http_expr_functions;
mod http;
pub use http::{HTTPBody, HTTPRewrite};

// Loads the logic for PHP based custom uploaders.
mod php_bootstrapping;
mod php;

// Exports the JSON structure for custom uploaders.
mod config_structure;
pub use config_structure::{CustomUploader, IntoUploader};
