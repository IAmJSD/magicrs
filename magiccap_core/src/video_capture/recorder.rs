use crate::region_selector::Region;
use std::sync::atomic::{AtomicBool, Ordering};
use xcap::Monitor;

#[cfg(target_os = "linux")]
use crate::video_capture::linux_recorder::{
    PlatformSpecificGIFRecorder, PlatformSpecificMP4Recorder,
};

enum RecorderType {
    MP4(PlatformSpecificMP4Recorder),
    GIF(PlatformSpecificGIFRecorder),
}

macro_rules! action {
    ($self:ident, $method_name:ident) => {
        match &$self.recorder {
            RecorderType::MP4(x) => x.$method_name(),
            RecorderType::GIF(x) => x.$method_name(),
        }
    };
}

pub struct Recorder {
    is_done: AtomicBool,
    recorder: RecorderType,
}

impl Recorder {
    pub fn new(gif: bool, monitor: Monitor, region: Region) -> Self {
        Self {
            is_done: AtomicBool::new(false),
            recorder: if gif {
                RecorderType::GIF(PlatformSpecificGIFRecorder::new(monitor, region))
            } else {
                RecorderType::MP4(PlatformSpecificMP4Recorder::new(monitor, region))
            },
        }
    }

    pub fn wait_for_stop(&self) {
        action!(self, wait_for_stop)
    }

    pub fn stop_record_thread(&self) {
        // Ensure this is a unique usage.
        if self.is_done.swap(true, Ordering::AcqRel) {
            return;
        }

        action!(self, stop_record_thread)
    }

    pub fn wait_for_encoding(&self) -> Vec<u8> {
        action!(self, wait_for_encoding)
    }
}
