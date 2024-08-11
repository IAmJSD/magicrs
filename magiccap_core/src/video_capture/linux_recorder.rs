use crate::region_selector::Region;
use xcap::Monitor;

pub struct PlatformSpecificMP4Recorder {}

impl PlatformSpecificMP4Recorder {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        Self {}
    }

    pub fn stop_record_thread(&self) {
        // TODO
    }

    pub fn wait_for_encoding_thread(&self) -> Vec<u8> {
        // TODO
        Vec::new()
    }
}

pub struct PlatformSpecificGIFRecorder {}

impl PlatformSpecificGIFRecorder {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        Self {}
    }

    pub fn stop_record_thread(&self) {
        // TODO
    }

    pub fn wait_for_encoding_thread(&self) -> Vec<u8> {
        // TODO
        Vec::new()
    }
}
