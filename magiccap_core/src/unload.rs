use crate::{database, hotkeys, search_indexing, statics};

pub fn application_unload() {
    // Drop all hotkeys.
    hotkeys::drop_all_hotkeys();

    // Activate the kill switch.
    statics::KILL_SWITCH.store(true, std::sync::atomic::Ordering::Relaxed);

    // Disconnect the database.
    database::disconnect();

    // Destroy the thread pool.
    statics::kill_thread_pool();

    // Disconnect the search index.
    search_indexing::disconnect_index();
}
