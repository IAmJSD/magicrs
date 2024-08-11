use crate::region_selector::Region;
use std::sync::atomic::{AtomicBool, Ordering};
use xcap::Monitor;

pub struct Recorder {
    is_done: AtomicBool,
}

impl Recorder {
    pub fn new(gif: bool, monitor: Monitor, region: Region) -> Self {
        // TODO
        Self {
            is_done: AtomicBool::new(false),
        }
    }

    pub fn stop_record_thread(&self) {
        // Ensure this is a unique usage.
        if self.is_done.swap(true, Ordering::AcqRel) {
            return;
        }

        // TODO
    }

    pub fn wait_for_encoding_thread(&self) -> Vec<u8> {
        // TODO
        Vec::new()
    }
}
