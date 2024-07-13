use std::sync::Arc;
use base64::Engine;
use evalexpr::ContextWithMutableFunctions;

macro_rules! func {
    ($expr_map:expr, $name:expr, $fn:expr) => {
        $expr_map.set_function(
            $name.to_string(), evalexpr::Function::new($fn),
        ).unwrap();
    }
}

fn serde2evalexpr(value: serde_json::Value) -> Result<evalexpr::Value, evalexpr::EvalexprError> {
    match value {
        serde_json::Value::Null => Ok(evalexpr::Value::Empty),
        serde_json::Value::Bool(b) => Ok(evalexpr::Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            Ok(if n.is_i64() {
                evalexpr::Value::Int(n.as_i64().unwrap())
            } else {
                evalexpr::Value::Float(n.as_f64().unwrap())
            })
        },
        serde_json::Value::String(s) => Ok(evalexpr::Value::String(s)),
        serde_json::Value::Array(arr) => {
            let mut vec = Vec::new();
            for v in arr {
                vec.push(match serde2evalexpr(v) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                });
            }
            Ok(evalexpr::Value::Tuple(vec))
        },
        _ => Err(evalexpr::EvalexprError::CustomMessage(
            "The value type is not supported.".to_string(),
        )),
    }
}

fn process_json_path(mut body: serde_json::Value, path: String) -> Result<serde_json::Value, String> {
    let mut current = &mut body;
    for part in path.split('.') {
        // If current is an array, try to parse the part as an index.
        if let Ok(i) = part.parse::<usize>() {
            let arr = match current.as_array_mut() {
                Some(a) => a,
                None => return Err("The body is not an array.".to_string()),
            };
            if i >= arr.len() {
                return Err(format!("The index {} is out of bounds.", i));
            }
            current = &mut arr[i];
            continue;
        }

        // Otherwise, handle this as a key.
        current = match current.get_mut(part) {
            Some(v) => v,
            None => return Err(format!("The path part {} does not exist.", part)),
        };
    }
    Ok(current.take())
}

pub fn add_default_functions(
    expr_map: &mut evalexpr::HashMapContext, body_arc: Arc<Vec<u8>>,
) {
    // Add json_path.
    let ar = body_arc.clone();
    func!(
        expr_map, "json_path", move |arg| {
            // Parse the JSON.
            let mut body = match serde_json::from_slice(&ar) {
                Ok(b) => b,
                Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                    format!("Failed to parse the body as JSON: {}.", e),
                )),
            };

            // Check if the argument is none. If so, just try to parse the body.
            if arg.is_empty() {
                return serde2evalexpr(body);    
            }

            // Check if the argument is a number and if so try to index the body.
            if let Ok(i) = arg.as_int() {
                let  arr = match body.as_array_mut() {
                    Some(a) => a,
                    None => return Err(evalexpr::EvalexprError::CustomMessage(
                        "The body is not an array.".to_string(),
                    )),
                };
                if i < 0 || i >= arr.len() as i64 {
                    return Err(evalexpr::EvalexprError::CustomMessage(
                        format!("The index {} is out of bounds.", i),
                    ));
                }
                return serde2evalexpr(arr[i as usize].take());
            }

            // Check if the argument is a string and if so handle it as a path.
            let path = match arg.as_string() {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            let path_result = match process_json_path(body, path) {
                Ok(v) => v,
                Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                    format!("Failed to process the JSON path: {}.", e),
                )),
            };

            // Convert the result to an evalexpr value.
            serde2evalexpr(path_result)
        }
    );

    // Add xml_path.
    let ar = body_arc.clone();
    func!(
        expr_map, "xml_path", move |arg| {
            // Process the body as a string.
            let body = match std::str::from_utf8(&ar) {
                Ok(s) => s,
                Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                    format!("Failed to convert the body to a string: {}.", e),
                )),
            };

            // Make sure arg is a string.
            let path = match arg.as_string() {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
    
            // Parse the XML.
            let package = match sxd_document::parser::parse(body) {
                Ok(p) => p,
                Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                    format!("Failed to parse the body as XML: {}.", e),
                )),
            };
            let document = package.as_document();

            // Get the value.
            let value = match sxd_xpath::evaluate_xpath(&document, &path) {
                Ok(v) => v,
                Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                    format!("Failed to evaluate the XML path: {}.", e),
                )),
            };

            // Figure out what to return.
            match value {
                sxd_xpath::Value::Nodeset(nodes) => {
                    let mut vec = Vec::new();
                    for node in nodes {
                        vec.push(evalexpr::Value::String(node.string_value()));
                    }
                    Ok(evalexpr::Value::Tuple(vec))
                },
                sxd_xpath::Value::Boolean(b) => Ok(evalexpr::Value::Boolean(b)),
                sxd_xpath::Value::Number(n) => Ok(evalexpr::Value::Float(n)),
                sxd_xpath::Value::String(s) => Ok(evalexpr::Value::String(s)),
            }
        }
    );

    // Add base64_encode.
    func!(
        expr_map, "base64_encode", |arg| {
            let str = match arg.as_string() {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            Ok(
                evalexpr::Value::String(base64::engine::general_purpose::STANDARD.encode(&str.as_bytes())),
            )
        }
    );

    // Add base64_decode.
    func!(
        expr_map, "base64_decode", |arg| {
            let str = match arg.as_string() {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            let decoded = match base64::engine::general_purpose::STANDARD.decode(&str) {
                Ok(d) => d,
                Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                    format!("Failed to decode the base64 string: {}.", e),
                )),
            };
            Ok(evalexpr::Value::String(
                match std::str::from_utf8(&decoded) {
                    Ok(s) => s.to_string(),
                    Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                        format!("Failed to convert the base64 string to a string: {}.", e),
                    )),
                },
            ))
        }
    );

    // Add string_body.
    func!(
        expr_map, "string_body", move |_| {
            let body_arc = body_arc.clone();
            let body = match std::str::from_utf8(&body_arc) {
                Ok(s) => s,
                Err(e) => return Err(evalexpr::EvalexprError::CustomMessage(
                    format!("Failed to convert the body to a string: {}.", e),
                )),
            };
            Ok(evalexpr::Value::String(body.to_string()))
        }
    );

    // Add url_encode.
    func!(
        expr_map, "url_encode", |arg| {
            let str = match arg.as_string() {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            Ok(
                evalexpr::Value::String(urlencoding::encode(&str).to_string()),
            )
        }
    );
}
