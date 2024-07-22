use std::{collections::HashMap, str::FromStr, sync::Arc};
use base64::{engine::general_purpose, Engine};
use evalexpr::{ContextWithMutableFunctions, EvalexprError, Value};
use mime::Mime;
use serde::{Deserialize, Serialize};
use super::http_expr_functions::add_default_functions;

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum HTTPRewrite {
    Config(String),
    Static(String),
    Filename,
    MIME,
}

fn rewrite_processor(
    rewrites: &HashMap<String, HTTPRewrite>, value: &str, filename: &str,
    mime_type: &str, config: &HashMap<String, serde_json::Value>,
) -> String {
    let mut value = value.to_string();
    for (key, rewrite) in rewrites {
        let replacement = match rewrite {
            HTTPRewrite::Config(s) => {
                match config.get(s) {
                    Some(v) => {
                        match v {
                            serde_json::Value::String(s) => s.to_string(),
                            _ => v.to_string(),
                        }
                    },
                    None => "".to_string(),
                }
            },
            HTTPRewrite::Static(s) => s.clone(),
            HTTPRewrite::Filename => filename.to_string(),
            HTTPRewrite::MIME => mime_type.to_string(),
        };
        value = value.replace(key, replacement.as_str());
    }
    value
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum URLEncodingType {
    Hex, B64URL, B64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct URLEncodingOpts {
    pub name: String,
    pub encoding_type: URLEncodingType,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum HTTPBody {
    Raw,
    URLEncoded(HashMap<String, String>, URLEncodingOpts),
    MultipartForm(HashMap<String, String>, String),
}

fn consume_all_reader(
    mut reader: Box<dyn std::io::Read + Send + Sync>
) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    match reader.read_to_end(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => Err(e.to_string()),
    }
}

fn process_http_response(
    template: &str, filename: &str, mime_type: &str,
    rewrites: HashMap<String, HTTPRewrite>, http_response: ureq::Response,
    config: HashMap<String, serde_json::Value>,
) -> Result<String, String> {
    // Get the expr map.
    let mut expr_map = evalexpr::HashMapContext::new();
    let filename_str = filename.to_string();
    let mime_str = mime_type.to_string();
    let rewrites_arc = Arc::new(rewrites);
    let config_arc = Arc::new(config);
    expr_map.set_function(
        "get_rewrite".to_string(),
        evalexpr::Function::new(move |arg| {
            let rewrites_ref = rewrites_arc.clone();
            let str = match arg.as_string() {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            let rewrite = match rewrites_ref.get(str.as_str()) {
                Some(r) => r,
                None => return Err(EvalexprError::CustomMessage(
                    format!("The rewrite {} does not exist.", str),
                )),
            };
            match rewrite {
                HTTPRewrite::Config(s) => {
                    match config_arc.get(s) {
                        Some(v) => {
                            match v {
                                serde_json::Value::String(s) => Ok(Value::String(s.to_string())),
                                _ => Ok(Value::String(v.to_string())),
                            }
                        },
                        None => Ok(Value::String("".to_string())),
                    }
                },
                HTTPRewrite::Static(s) => Ok(Value::String(s.to_string())),
                HTTPRewrite::Filename => Ok(Value::String(filename_str.clone())),
                HTTPRewrite::MIME => Ok(Value::String(mime_str.clone())),
            }
        })).unwrap();
    let mut headers = HashMap::new();
    for key in http_response.headers_names() {
        let key_cpy = key.clone();
        headers.insert(key_cpy,
            http_response.header(key.as_str()).unwrap().to_string());
    }
    expr_map.set_function(
        "get_header".to_string(),
        evalexpr::Function::new(move |arg| {
            let key = match arg.as_string() {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            match headers.get(key.as_str()) {
                Some(v) => Ok(Value::String(v.to_string())),
                None => Ok(Value::String("".to_string())),
            }
        }),
    ).unwrap();
    let body_arc = match consume_all_reader(http_response.into_reader()) {
        Ok(v) => Arc::new(v),
        Err(e) => return Err(e),
    };
    add_default_functions(&mut expr_map, body_arc);

    // Call the template.
    match evalexpr::eval_with_context(template, &expr_map) {
        Ok(v) => match v.as_string() {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn http(
    filename: &str, mime_type: &str,
    rewrites: HashMap<String, HTTPRewrite>, url_template: &str,
    method: &str, header_templates: HashMap<String, String>, body: HTTPBody,
    config: HashMap<String, serde_json::Value>,
    mut reader: Box<dyn std::io::Read + Send + Sync>, response: &str,
) -> Result<String, String> {
    // Rewrite the URL.
    let url = rewrite_processor(&rewrites, url_template, filename, mime_type, &config);

    // Build the request.
    let mut req = ureq::agent().request(method, url.as_str());
    for (key, value) in header_templates {
        // Rewrite the header value.
        let key_rewritten = rewrite_processor(
            &rewrites, key.as_str(), filename, mime_type, &config,
        );
        let value_rewritten = rewrite_processor(
            &rewrites, value.as_str(), filename, mime_type, &config,
        );

        // Add the header.
        req = req.set(key_rewritten.as_str(), value_rewritten.as_str());
    }

    // Add the body and make the call.
    let result = match body {
        HTTPBody::Raw => {
            if !req.has("content-type") {
                // Set the content type if it is not set.
                req = req.set("content-type", mime_type);
            }
            req.send(reader)
        },
        HTTPBody::MultipartForm(
            other_items, body_name,
        ) => {
            // Build the multipart form.
            let mut multipart_builder = ureq_multipart::MultipartBuilder::new();
            for (key, value) in other_items {
                multipart_builder = multipart_builder.add_text(key.as_str(), value.as_str()).unwrap();
            }
            multipart_builder = match multipart_builder.add_stream(
                &mut reader, body_name.as_str(), Some(filename),
                Some(Mime::from_str(mime_type).unwrap()),
            ) {
                Ok(m) => m,
                Err(e) => return Err(e.to_string()),
            };

            // Finish the multipart data.
            let (content_type, multipart) = match multipart_builder.finish() {
                Ok(m) => m,
                Err(err) => return Err(err.to_string()),
            };
            req = req.set("content-type", &content_type);

            // Send the request.
            req.send_bytes(&multipart)
        },
        HTTPBody::URLEncoded(
            other_items, field,
        ) => {
            // Get the body.
            let body = match consume_all_reader(reader) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            // Get the query items.
            let mut query: Vec<(String, String)> = other_items
                .into_iter().collect::<Vec<(String, String)>>();

            // Figure out how to handle the body.
            let key = field.name;
            let value = match field.encoding_type {
                URLEncodingType::Hex => body.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
                URLEncodingType::B64URL => general_purpose::URL_SAFE.encode(body),
                URLEncodingType::B64 => general_purpose::STANDARD.encode(body),
            };

            // Add the body to the query and then sort.
            query.push((key, value));
            query.sort_by(|a, b| a.0.cmp(&b.0));

            // Send the request.
            if method == "GET" {
                req.query_pairs(
                    query.iter().map(|(k, v)| {(k.as_str(), v.as_str())})
                ).send_string("")
            } else {
                // Build the query string.
                let query_string = query.iter().map(|(k, v)| {
                    format!("{}={}", urlencoding::encode(k), urlencoding::encode(v))
                }).collect::<Vec<String>>().join("&");

                // Send the request.
                req.send_string(query_string.as_str())
            }
        },
    };

    // Handle the result.
    match result {
        Ok(v) => process_http_response(
            response, filename, mime_type, rewrites, v,
            config,
        ),
        Err(e) => Err(e.to_string()),
    }
}
