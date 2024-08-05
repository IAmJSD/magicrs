use crate::statics::CONFIG_FOLDER;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tantivy::{
    schema::{self, Value},
    Index, IndexWriter, TantivyDocument, Term,
};

// Defines the schema for the index.
static SCHEMA: Lazy<schema::Schema> = Lazy::new(|| {
    let mut schema = schema::Schema::builder();
    schema.add_i64_field("capture_id", schema::STORED | schema::INDEXED);
    schema.add_text_field("filename", schema::TEXT);
    schema.add_text_field("text", schema::TEXT);
    schema.add_text_field("window_names_space_joined", schema::TEXT);
    schema.build()
});

// Build the index.
static mut INDEX: Lazy<Option<Index>> = Lazy::new(|| {
    // Get the folder to store the index.
    let folder = CONFIG_FOLDER.join("search");

    // Make sure the folder exists.
    std::fs::create_dir_all(&folder).unwrap();

    // Try to open the index.
    if let Ok(index) = Index::open_in_dir(&folder) {
        return Some(index);
    }

    // Create the index and return it.
    Some(Index::create_in_dir(folder, SCHEMA.clone()).unwrap())
});

// Defines an index writer to be shared across threads.
static INDEX_WRITER: Lazy<Mutex<Option<IndexWriter>>> = Lazy::new(|| {
    let index = match unsafe { &*INDEX } {
        Some(index) => index,
        None => return Mutex::new(None),
    };
    Mutex::new(Some(index.writer(50_000_000).unwrap()))
});

// Disconnects the index by dropping it. Used on unload.
pub fn disconnect_index() {
    unsafe {
        *INDEX = None;
    }
    let mut lock = INDEX_WRITER.lock().unwrap();
    *lock = None;
}

// Write a capture into the index.
pub fn insert_capture(capture_id: i64, filename: &str, text: String, window_names: Vec<String>) {
    let mut guard = INDEX_WRITER.lock().unwrap();
    let writer_ref = match guard.as_mut() {
        Some(writer) => writer,
        None => return,
    };
    let mut doc = TantivyDocument::new();
    doc.add_i64(SCHEMA.get_field("capture_id").unwrap(), capture_id);
    doc.add_text(SCHEMA.get_field("filename").unwrap(), filename);
    doc.add_text(SCHEMA.get_field("text").unwrap(), text);
    doc.add_text(
        SCHEMA.get_field("window_names_space_joined").unwrap(),
        window_names.join(" "),
    );
    writer_ref.add_document(doc).unwrap();
    writer_ref.commit().unwrap();
}

// Remove a capture from the index if it exists.
pub fn remove_capture(capture_id: i64) {
    let mut guard = INDEX_WRITER.lock().unwrap();
    let writer_ref = match guard.as_mut() {
        Some(writer) => writer,
        None => return,
    };
    writer_ref.delete_term(Term::from_field_i64(
        SCHEMA.get_field("capture_id").unwrap(),
        capture_id,
    ));
    writer_ref.commit().unwrap();
}

// Search the index for captures that match the query.
pub fn search_index(query: &str) -> Vec<i64> {
    let index = match unsafe { &*INDEX } {
        Some(index) => index,
        None => return Vec::new(),
    };
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query_parser = tantivy::query::QueryParser::for_index(
        &index,
        vec![
            SCHEMA.get_field("filename").unwrap(),
            SCHEMA.get_field("text").unwrap(),
            SCHEMA.get_field("window_names_space_joined").unwrap(),
        ],
    );
    let query = match query_parser.parse_query(query) {
        Ok(query) => query,
        Err(_) => return Vec::new(),
    };
    let top_docs = searcher
        .search(&query, &tantivy::collector::TopDocs::with_limit(10))
        .unwrap();
    top_docs
        .into_iter()
        .map(|(_, doc_address)| {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).unwrap();
            let capture_id = retrieved_doc
                .get_first(SCHEMA.get_field("capture_id").unwrap())
                .unwrap()
                .as_i64()
                .unwrap();
            capture_id
        })
        .collect()
}

// Wipes the index clean.
pub fn wipe_index() {
    let mut guard = INDEX_WRITER.lock().unwrap();
    let writer_ref = match guard.as_mut() {
        Some(writer) => writer,
        None => return,
    };
    writer_ref.delete_all_documents().unwrap();
    writer_ref.commit().unwrap();
}
