use serde::Serialize;
use crate::database;

// Defines an API error.
#[derive(Serialize)]
struct APIError {
    message: String,
    user_facing: bool,
}

// Turn a method that returns only an error into a Result.
fn err_only(err: Option<APIError>) -> Result<serde_json::Value, APIError> {
    match err {
        Some(err) => Err(err),
        None => Ok(serde_json::Value::Null),
    }
}

// Used to get the ID from the query.
fn ensure_id_number(id: Option<&str>) -> Result<i64, APIError> {
    // Make sure the ID is not none.
    let id = match id {
        Some(id) => id,
        None => return Err(APIError {
            message: "The id is required.".to_string(),
            user_facing: true,
        }),
    };

    // Convert the ID to an integer.
    match id.parse::<i64>() {
        Ok(id) => Ok(id),
        Err(_) => Err(APIError {
            message: "The id is not a valid integer.".to_string(),
            user_facing: true,
        }),
    }
}

// Allows you to delete a capture.
fn delete_capture(id: Option<&str>) -> Option<APIError> {
    let id = match ensure_id_number(id) {
        Ok(id) => id, Err(err) => return Some(err),
    };
    match database::delete_capture(id) {
        true => None,
        false => Some(APIError {
            message: "The capture does not exist.".to_string(),
            user_facing: true,
        }),
    }
}

// Opens a capture URL in the default browser.
fn open_url(id: Option<&str>) -> Option<APIError> {
    let id = match ensure_id_number(id) {
        Ok(id) => id, Err(err) => return Some(err),
    };

    let capture = match database::get_capture(id) {
        Some(capture) => capture,
        None => return Some(APIError {
            message: "The capture does not exist.".to_string(),
            user_facing: true,
        }),
    };

    let uri = match capture.url {
        Some(url) => url,
        None => return Some(APIError {
            message: "The capture does not have a URL.".to_string(),
            user_facing: true,
        }),
    };
    let uri = match uriparse::URI::try_from(uri.as_str()) {
        Ok(uri) => uri,
        Err(_) => return Some(APIError {
            message: "The capture URL is not valid.".to_string(),
            user_facing: true,
        }),
    };

    // Check if the protocol is not HTTP or HTTPS.
    if uri.scheme() != "http" && uri.scheme() != "https" {
        return Some(APIError {
            message: "The capture URL is not HTTP or HTTPS.".to_string(),
            user_facing: true,
        });
    }

    // Open the URL in the default browser.
    open::that(uri.to_string()).unwrap();

    // No errors!
    None
}

// Opens a capture file. We can be a bit more trusting because this had to be a file on the system.
fn open_file(id: Option<&str>) -> Option<APIError> {
    let id = match ensure_id_number(id) {
        Ok(id) => id, Err(err) => return Some(err),
    };

    let capture = match database::get_capture(id) {
        Some(capture) => capture,
        None => return Some(APIError {
            message: "The capture does not exist.".to_string(),
            user_facing: true,
        }),
    };

    let file_path = match capture.file_path {
        Some(file_path) => file_path,
        None => return Some(APIError {
            message: "The capture does not have a file path.".to_string(),
            user_facing: true,
        }),
    };

    // Open the file.
    open::that(file_path).unwrap();

    // No errors!
    None
}

// Shows the capture in the folder.
fn show_in_folder(id: Option<&str>) -> Option<APIError> {
    let id = match ensure_id_number(id) {
        Ok(id) => id, Err(err) => return Some(err),
    };

    let capture = match database::get_capture(id) {
        Some(capture) => capture,
        None => return Some(APIError {
            message: "The capture does not exist.".to_string(),
            user_facing: true,
        }),
    };

    let file_path = match capture.file_path {
        Some(file_path) => file_path,
        None => return Some(APIError {
            message: "The capture does not have a file path.".to_string(),
            user_facing: true,
        }),
    };

    // Split the directory from the file path in a OS agnostic way.
    let dir = match std::path::Path::new(&file_path).parent() {
        Some(dir) => dir,
        None => return Some(APIError {
            message: "The capture file path is not valid.".to_string(),
            user_facing: true,
        }),
    };

    // Check if the directory exists and is a directory.
    if !dir.exists() || !dir.is_dir() {
        return Some(APIError {
            message: "The capture file path is not a directory.".to_string(),
            user_facing: true,
        });
    }

    // Open the directory in the default file manager.
    open::that(dir).unwrap();

    // No errors!
    None
}

// Gets a capture URL.
fn get_url(id: Option<&str>) -> Result<serde_json::Value, APIError> {
    let id = match ensure_id_number(id) {
        Ok(id) => id, Err(err) => return Err(err),
    };

    let capture = match database::get_capture(id) {
        Some(capture) => capture,
        None => return Err(APIError {
            message: "The capture does not exist.".to_string(),
            user_facing: true,
        }),
    };

    match capture.url {
        Some(url) => Ok(serde_json::Value::String(url)),
        None => Ok(serde_json::Value::Null),
    }
}

// find a string option in a JSON object.
fn query_find<'q>(query: &'q serde_json::Value, key: &str) -> Option<&'q str> {
    let v = match query.get(key) {
        Some(v) => v,
        None => return None,
    };
    match v.as_str() {
        Some(v) => Some(v),
        None => None,
    }
}

// Sets a configuration option.
fn set_config_option(query: &serde_json::Value) -> Option<APIError> {
    // TODO: reload things here!

    let key = match query_find(query, "key") {
        Some(key) => key,
        None => return Some(APIError {
            message: "The key is required.".to_string(),
            user_facing: true,
        }),
    };

    // Handle if query["value"] is not in the JSON object as any type.
    let value = match query.get("value") {
        Some(value) => value,
        None => return Some(APIError {
            message: "The value is required.".to_string(),
            user_facing: true,
        }),
    };

    database::set_config_option(key, value);

    // No errors!
    None
}

// Deletes a configuration option.
fn delete_config_option(key: Option<&str>) -> Option<APIError> {
    let key = match key {
        Some(key) => key,
        None => return Some(APIError {
            message: "The key is required.".to_string(),
            user_facing: true,
        }),
    };

    database::delete_config_option(key);

    // No errors!
    None
}

// Gets a configuration option.
fn get_config_option(key: Option<&str>) -> Result<serde_json::Value, APIError> {
    let key = match key {
        Some(key) => key,
        None => return Err(APIError {
            message: "The key is required.".to_string(),
            user_facing: true,
        }),
    };

    match database::get_config_option(key) {
        Some(value) => Ok(value),
        None => Ok(serde_json::Value::Null),
    }
}

// Sets an uploader configuration option.
fn set_uploader_config_option(query: &serde_json::Value) -> Option<APIError> {
    let uploader_id = match query_find(query, "uploaderId") {
        Some(uploader_id) => uploader_id,
        None => return Some(APIError {
            message: "uploaderId is required.".to_string(),
            user_facing: true,
        }),
    };

    let key = match query_find(query, "key") {
        Some(key) => key,
        None => return Some(APIError {
            message: "The key is required.".to_string(),
            user_facing: true,
        }),
    };

    let value = match query.get("value") {
        Some(value) => value,
        None => return Some(APIError {
            message: "The value is required.".to_string(),
            user_facing: true,
        }),
    };

    database::set_uploader_config_item(uploader_id, key, value);

    // No errors!
    None
}

// Deletes an uploader configuration option.
fn delete_uploader_config_option(query: &serde_json::Value) -> Option<APIError> {
    let uploader_id = match query_find(query, "uploaderId") {
        Some(uploader_id) => uploader_id,
        None => return Some(APIError {
            message: "uploaderId is required.".to_string(),
            user_facing: true,
        }),
    };

    let key = match query_find(query, "key") {
        Some(key) => key,
        None => return Some(APIError {
            message: "The key is required.".to_string(),
            user_facing: true,
        }),
    };

    database::delete_uploader_config_item(uploader_id, key);

    // No errors!
    None
}

// Gets an uploader configuration options.
fn get_uploader_config_options(uploader_id: Option<&str>) -> Result<serde_json::Value, APIError> {
    let uploader_id = match uploader_id {
        Some(uploader_id) => uploader_id,
        None => return Err(APIError {
            message: "uploaderId is required.".to_string(),
            user_facing: true,
        }),
    };

    Ok(
        serde_json::Value::Object(
            serde_json::Map::from_iter(
                database::get_uploader_config_items(uploader_id).
                    into_iter().
                    map(|(k, v)| (k, v)),
            ),
        ),
    )
}

// Selects a folder on macOS.
#[cfg(target_os = "macos")]
fn select_folder() -> Result<serde_json::Value, APIError> {
    use cacao::foundation::NSURL;
    use objc::runtime::Object;
    use crate::macos;

    let string_id: usize = unsafe { macos::open_file_dialog(true) };
    if string_id == 0 {
        return Ok(serde_json::Value::Null);
    }
    let url = NSURL::retain(string_id as *mut Object);

    let path = url.pathbuf();
    if !path.exists() || !path.is_dir() {
        return Ok(serde_json::Value::Null);
    }

    Ok(serde_json::Value::String(path.to_str().unwrap().to_string()))
}

// Selects a folder on Linux and Windows.
#[cfg(not(target_os = "macos"))]
fn select_folder() -> Result<serde_json::Value, APIError> {
    use native_dialog::FileDialog;

    let res = FileDialog::new().show_open_single_dir().unwrap();
    match res {
        None => Ok(serde_json::Value::Null),
        Some(fp) => Ok(serde_json::Value::String(fp.to_str().unwrap().to_string())),
    }
}

// Selects a file on macOS.
#[cfg(target_os = "macos")]
fn select_file() -> Result<serde_json::Value, APIError> {
    use cacao::foundation::NSURL;
    use objc::runtime::Object;
    use crate::macos;

    let string_id: usize = unsafe { macos::open_file_dialog(false) };
    if string_id == 0 {
        return Ok(serde_json::Value::Null);
    }
    let url = NSURL::retain(string_id as *mut Object);

    let path = url.pathbuf();
    if !path.exists() || !path.is_file() {
        return Ok(serde_json::Value::Null);
    }

    // Read the file.
    let data = match std::fs::read(&path) {
        Ok(data) => data,
        Err(_) => return Ok(serde_json::Value::Null),
    };

    // Turn the vec into a string.
    let data = match String::from_utf8(data) {
        Ok(data) => data,
        Err(_) => return Ok(serde_json::Value::Null),
    };

    Ok(serde_json::Value::String(data))
}

// Select a file in Linux and Windows.
#[cfg(not(target_os = "macos"))]
fn select_file() -> Result<serde_json::Value, APIError> {
    use native_dialog::FileDialog;

    let res = FileDialog::new().show_open_single_file().unwrap();
    match res {
        None => Ok(serde_json::Value::Null),
        Some(path) => {
            // Read the file.
            let data = match std::fs::read(&path) {
                Ok(data) => data,
                Err(_) => return Ok(serde_json::Value::Null),
            };

            // Turn the vec into a string.
            let data = match String::from_utf8(data) {
                Ok(data) => data,
                Err(_) => return Ok(serde_json::Value::Null),
            };

            // Return as a string.
            Ok(serde_json::Value::String(data))
        },
    }
}

// Get all the uploaders.
fn get_uploaders() -> Result<serde_json::Value, APIError> {
    Ok(
        serde_json::Value::Object(
            serde_json::Map::from_iter(
                crate::uploaders::UPLOADERS.iter().
                    map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap())),
            ),
        )
    )
}

// Starts the hotkey capture.
fn start_hotkey_capture() -> Option<APIError> {
    // TODO
    None
}

// Stops the hotkey capture.
fn stop_hotkey_capture() -> Option<APIError> {
    // TODO
    None
}

static UPLOAD_TEST_IMAGE: &[u8] = include_bytes!("./upload_test_image.png");

// Allows you to test a uploader.
fn test_uploader(id: Option<&str>) -> Option<APIError> {
    let id = match id {
        Some(id) => id,
        None => return Some(APIError {
            message: "The id is required.".to_string(),
            user_facing: true,
        }),
    };

    let reader = Box::new(
        std::io::Cursor::new(UPLOAD_TEST_IMAGE)
    );
    match crate::uploaders::call_uploader(
        id, reader, "test.png",
    ) {
        Ok(_) => None,
        Err(e) => Some(APIError {
            message: format!("The uploader failed: {}", e),
            user_facing: true,
        }),
    }
}

// Routes the API call to the correct function.
fn route_api_call(api_type: &str, query: &serde_json::Value) -> Result<serde_json::Value, APIError> {
    match api_type {
        // Handle deleting a capture.
        "delete_capture" => err_only(delete_capture(query_find(query, "id"))),

        // Opens a capture URL.
        "open_url" => err_only(open_url(query_find(query, "id"))),

        // Opens a capture file.
        "open_file" => err_only(open_file(query_find(query, "id"))),

        // Shows the capture in the folder.
        "show_in_folder" => err_only(show_in_folder(query_find(query, "id"))),

        // Sets a configuration option.
        "set_config_option" => err_only(set_config_option(query)),

        // Deletes a configuration option.
        "delete_config_option" => err_only(delete_config_option(query_find(query, "key"))),

        // Gets a configuration option.
        "get_config_option" => get_config_option(query_find(query, "key")),

        // Sets a uploader configuration option.
        "set_uploader_config_option" => err_only(set_uploader_config_option(query)),

        // Deletes a uploader configuration option.
        "delete_uploader_config_option" => err_only(delete_uploader_config_option(query)),

        // Gets a uploaders configuration options.
        "get_uploader_config_options" => get_uploader_config_options(query_find(query, "uploaderId")),

        // Gets a capture URL.
        "get_url" => get_url(query_find(query, "id")),

        // Selects a folder.
        "select_folder" => select_folder(),

        // Selects a file.
        "select_file" => select_file(),

        // Get all the uploaders.
        "get_uploaders" => get_uploaders(),

        // Starts the hotkey capture.
        "start_hotkey_capture" => err_only(start_hotkey_capture()),

        // Stops the hotkey capture.
        "stop_hotkey_capture" => err_only(stop_hotkey_capture()),

        // Allows you to test a uploader.
        "test_uploader" => err_only(test_uploader(query_find(query, "id"))),

        // Catch all unknown API types.
        _ => Err(APIError {
            message: "Unknown API type".to_string(),
            user_facing: false,
        }),
    }
}

// The low level API response structure.
#[derive(Serialize)]
struct APIResponse {
    err: Option<APIError>,
    data: serde_json::Value,
}

// The main export to handle the low level API calls.
pub fn handle_api_call(query: serde_json::Value) -> Vec<u8> {
    let api_type = query["_t"].as_str().unwrap();
    match route_api_call(api_type, &query) {
        Ok(v) => serde_json::to_vec(&APIResponse {
            err: None,
            data: v,
        }).unwrap(),
        Err(err) => serde_json::to_vec(&APIResponse {
            err: Some(err),
            data: serde_json::Value::Null,
        }).unwrap(),
    }
}
