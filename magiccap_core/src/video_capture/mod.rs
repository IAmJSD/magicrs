mod gif_encoder;
mod recorder;

#[cfg(target_os = "linux")]
mod linux_recorder;

#[cfg(target_os = "linux")]
mod mp4_encoder;

use crate::{region_selector::Region, temp_icon::IconHandler};
use recorder::Recorder;
use std::sync::Arc;
use xcap::Monitor;

// Starts the video capturer.
pub fn start_recorder(gif: bool, monitor: Monitor, region: Region) -> Vec<u8> {
    // Start the recorder and temporary icon.
    let recorder_arc = Arc::new(Recorder::new(gif, monitor, region));
    let clone1 = Arc::clone(&recorder_arc);
    let mut temp_icon = IconHandler::new(Box::new(move || clone1.stop_record_thread()));

    // Wait for stop to be called.
    recorder_arc.wait_for_stop();

    // Tell the temporary icon to turn into a loading icon.
    temp_icon.processing();

    // Wait for encoding.
    let data = recorder_arc.wait_for_encoding();

    // Remove the temporary icon.
    temp_icon.remove();

    // Return the data we got from the recorder.
    data
}
