use crate::{config, search_indexing, statics::CONFIG_FOLDER};
use sqlite::{ConnectionThreadSafe, State};
use std::{borrow::Borrow, collections::HashMap, sync::RwLock};

// Defines the database connection.
static DATABASE: RwLock<Option<ConnectionThreadSafe>> = RwLock::new(None);

// Get the configurations for an uploader.
pub fn get_uploader_config_items(uploader_id: &str) -> HashMap<String, serde_json::Value> {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("SELECT name, value FROM uploader_config_items WHERE uploader_id = ?")
        .unwrap();

    // Execute the statement.
    stmt.bind((1, uploader_id)).unwrap();

    // Create the config items.
    let mut config_items = HashMap::new();
    while let Ok(State::Row) = stmt.next() {
        let name = stmt.read::<String, _>("name").unwrap();
        let value = stmt.read::<String, _>("value").unwrap();
        match serde_json::from_str(&value) {
            Ok(value) => {
                config_items.insert(name, value);
            }
            Err(_) => {}
        }
    }

    // Return the config items.
    config_items
}

// Sets a configuration option for an uploader.
pub fn set_uploader_config_item(uploader_id: &str, name: &str, value: &serde_json::Value) {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("INSERT OR REPLACE INTO uploader_config_items (uploader_id, name, value) VALUES (?, ?, ?)")
        .unwrap();

    // Create the binds.
    stmt.bind((1, uploader_id)).unwrap();
    stmt.bind((2, name)).unwrap();
    let s = value.to_string();
    stmt.bind((3, s.as_str())).unwrap();

    // Execute the statement.
    stmt.next().unwrap();

    // TODO: call any update hooks
}

// Deletes a configuration option for an uploader.
pub fn delete_uploader_config_item(uploader_id: &str, name: &str) {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("DELETE FROM uploader_config_items WHERE uploader_id = ? AND name = ?")
        .unwrap();

    // Create the binds.
    stmt.bind((1, uploader_id)).unwrap();
    stmt.bind((2, name)).unwrap();

    // Execute the statement.
    stmt.next().unwrap();

    // TODO: call any update hooks
}

// Gets a configuration option.
pub fn get_config_option(name: &str) -> Option<serde_json::Value> {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("SELECT value FROM config WHERE name = ?")
        .unwrap();

    // Get the value.
    stmt.bind((1, name)).unwrap();
    if let Ok(State::Row) = stmt.next() {
        let value = stmt.read::<String, _>("value").unwrap();
        let json: serde_json::Result<serde_json::Value> = serde_json::from_str(&value);
        if let Ok(json) = json {
            return Some(json);
        }
    }

    // Return none.
    None
}

// Sets a configuration option.
pub fn set_config_option(name: &str, value: &serde_json::Value) {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("INSERT OR REPLACE INTO config (name, value) VALUES (?, ?)")
        .unwrap();

    // Create the binds.
    stmt.bind((1, name)).unwrap();
    let s = value.to_string();
    stmt.bind((2, s.as_str())).unwrap();

    // Execute the statement.
    stmt.next().unwrap();

    // TODO: call any update hooks
}

// Deletes a configuration option.
pub fn delete_config_option(name: &str) {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("DELETE FROM config WHERE name = ?")
        .unwrap();

    // Create the binds.
    stmt.bind((1, name)).unwrap();

    // Execute the statement.
    stmt.next().unwrap();

    // TODO: call any update hooks
}

pub struct Capture {
    pub id: i64,
    pub created_at: String,
    pub success: bool,
    pub filename: String,
    pub file_path: Option<String>,
    pub url: Option<String>,
}

// Read a row into a capture.
fn read_capture(stmt: &sqlite::Statement) -> Capture {
    Capture {
        id: stmt.read::<i64, _>("id").unwrap(),
        created_at: stmt.read::<String, _>("created_at").unwrap(),
        success: stmt.read::<i64, _>("success").unwrap() == 1,
        filename: stmt.read::<String, _>("filename").unwrap(),
        file_path: stmt.read::<Option<String>, _>("file_path").unwrap(),
        url: stmt.read::<Option<String>, _>("url").unwrap(),
    }
}

// Gets a single capture from the database.
pub fn get_capture(id: i64) -> Option<Capture> {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare(
            "SELECT id, created_at, success, filename, file_path, url FROM captures WHERE id = ?",
        )
        .unwrap();

    // Execute the statement.
    stmt.bind((1, id)).unwrap();
    if let Ok(State::Row) = stmt.next() {
        return Some(read_capture(&stmt));
    }

    // Return none.
    None
}

// Get a vector of captures from the database. Returns all of the ones found.
pub fn get_many_captures(capture_ids: Vec<i64>) -> Vec<Capture> {
    // If the vector is empty, return an empty vector.
    if capture_ids.is_empty() {
        return Vec::new();
    }

    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Defines the query. This is safe because the ID's are all numbers.
    let ids = capture_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let query = format!(
        "SELECT id, created_at, success, filename, file_path, url FROM captures WHERE id IN ({})",
        ids
    );

    // Execute the statement.
    let mut stmt = database.prepare(query).unwrap();
    let mut captures = Vec::new();
    while let Ok(State::Row) = stmt.next() {
        captures.push(read_capture(&stmt));
    }

    // Return the captures.
    captures
}

// Gets the captures from the database.
pub fn get_captures() -> Vec<Capture> {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("SELECT id, created_at, success, filename, file_path, url FROM captures ORDER BY created_at DESC")
        .unwrap();

    // Execute the statement.
    let mut captures = Vec::new();
    while let Ok(State::Row) = stmt.next() {
        captures.push(read_capture(&stmt));
    }

    // Return the captures.
    captures
}

// Inserts a failed capture into the database.
pub fn insert_failed_capture(filename: &str, file_path: Option<&str>) {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare(
            "INSERT INTO captures (success, filename, file_path) VALUES (0, ?, ?) RETURNING id",
        )
        .unwrap();

    // Create the binds.
    stmt.bind((1, filename)).unwrap();
    stmt.bind((2, file_path)).unwrap();

    // Execute the statement.
    if let Ok(State::Row) = stmt.next() {
        let id: i64 = stmt.read(0).unwrap();
        config::update_webview_with_capture(id);
    }
}

// Inserts a successful capture into the database.
pub fn insert_successful_capture(
    filename: &str,
    file_path: Option<&str>,
    url: Option<&str>,
) -> i64 {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("INSERT INTO captures (success, filename, file_path, url) VALUES (1, ?, ?, ?) RETURNING id")
        .unwrap();

    // Create the binds.
    stmt.bind((1, filename)).unwrap();
    stmt.bind((2, file_path)).unwrap();
    stmt.bind((3, url)).unwrap();

    // Execute the statement.
    if let Ok(State::Row) = stmt.next() {
        let id: i64 = stmt.read(0).unwrap();
        config::update_webview_with_capture(id);
        return id;
    } else {
        panic!("Failed to insert successful capture into the database")
    }
}

// Deletes a capture from the database. Returns true if a capture was deleted or false if it never existed.
pub fn delete_capture(id: i64) -> bool {
    // Drop from the search index.
    search_indexing::remove_capture(id);

    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Prepare the statement.
    let mut stmt = database
        .prepare("DELETE FROM captures WHERE id = ?")
        .unwrap();

    // Execute the statement.
    stmt.bind((1, id)).unwrap();
    stmt.next().unwrap() == State::Done
}

// Does the database migrations.
fn do_migrations() {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Run the statements.
    let stmts = "
        CREATE TABLE IF NOT EXISTS uploader_config_items (
            uploader_id TEXT NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (uploader_id, name)
        );
        CREATE INDEX IF NOT EXISTS uploader_config_items_uploader_id ON uploader_config_items (uploader_id);

        CREATE TABLE IF NOT EXISTS config (
            name TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS captures (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success INTEGER NOT NULL,
            filename TEXT NOT NULL,
            file_path TEXT,
            url TEXT
        );
        CREATE INDEX IF NOT EXISTS captures_created_at_reversed ON captures (created_at DESC);
    ";
    database.execute(stmts).unwrap();
}

// Connects to the database.
pub fn connect() {
    // Lock the database.
    let mut database = DATABASE.write().unwrap();

    // Set the database to the connection.
    *database =
        Some(sqlite::Connection::open_thread_safe(CONFIG_FOLDER.join("database.db")).unwrap());

    // Drop the database lock.
    drop(database);

    // Do migrations.
    do_migrations();
}

// Disconnects from the database.
pub fn disconnect() {
    // Lock the database.
    let mut database = DATABASE.write().unwrap();

    // Set the database to none.
    *database = None;
}

// Wipe everything inside the database.
pub fn wipe_all() {
    // Acquire the database lock.
    let database_opt = DATABASE.read().unwrap();
    let database = database_opt.borrow().as_ref().unwrap();

    // Run the statements.
    let stmts = "
        DELETE FROM uploader_config_items;
        DELETE FROM config;
        DELETE FROM captures;
    ";
    database.execute(stmts).unwrap();

    // TODO: call any update hooks
}
