use crate::{region_selector::Region, statics::run_thread};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use xcap::Monitor;

use super::gif_encoder::GIFEncoder;

struct CaptureData {
    frame: Vec<u8>,
}

struct CaptureEnumerator {}

impl CaptureEnumerator {
    pub fn new(monitor: Monitor, region: Region, fps: u32, audio: bool) -> Self {
        Self {}
    }

    pub fn next(&mut self) -> Option<CaptureData> {
        // TODO
        None
    }
}

impl Drop for CaptureEnumerator {
    fn drop(&mut self) {
        // TODO: Disconnect!
    }
}

struct UIController {}

impl UIController {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        // TODO
        Self {}
    }

    pub fn new_frame(&mut self) {
        // TODO
    }
}

impl Drop for UIController {
    fn drop(&mut self) {
        // TODO: kill on main thread
    }
}

pub struct PlatformSpecificMP4Recorder {}

impl PlatformSpecificMP4Recorder {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        Self {}
    }

    pub fn stop_record_thread(&self) {
        // TODO
    }

    pub fn wait_for_encoding(&self) -> Vec<u8> {
        // TODO
        Vec::new()
    }
}

// Defines the structure that holds together all of the multi-threaded parts
// of the Linux GIF recording logic.
pub struct PlatformSpecificGIFRecorder {
    abort: Arc<AtomicBool>,
    encoder: Arc<Mutex<Option<GIFEncoder>>>,
    ui: Arc<Mutex<Option<UIController>>>,
}

impl PlatformSpecificGIFRecorder {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        // Create the structure and all of the Arc's.
        let encoder_arc = Arc::new(Mutex::new(Some(GIFEncoder::new(
            region.width,
            region.height,
            15,
        ))));
        let encoder_arc_clone = Arc::clone(&encoder_arc);
        let atom_arc = Arc::new(AtomicBool::new(false));
        let atom_arc_clone = Arc::clone(&atom_arc);
        let ui_arc = Arc::new(Mutex::new(Some(UIController::new(
            monitor.clone(),
            region.clone(),
        ))));
        let ui_arc_clone = Arc::clone(&ui_arc);
        let v = Self {
            abort: atom_arc,
            encoder: encoder_arc,
            ui: ui_arc,
        };

        // Create the recording thread.
        run_thread(move || {
            let mut lock = encoder_arc_clone.lock().unwrap();
            let guarded_value = lock.as_mut().unwrap();
            let mut e = CaptureEnumerator::new(monitor, region, 15, false);
            loop {
                if atom_arc_clone.load(Ordering::Relaxed) {
                    return;
                }
                match e.next() {
                    Some(v) => guarded_value.consume_rgba_frame(v.frame),
                    None => break,
                };
                if let Some(ui_controller) = ui_arc_clone.lock().unwrap().as_mut() {
                    ui_controller.new_frame();
                }
            }
        });

        // Return the structure for usage by the higher level APIs.
        v
    }

    pub fn stop_record_thread(&self) {
        // Drop the UI renderer and mark the recorder as aborted.
        self.ui.lock().unwrap().take();
        self.abort.store(true, Ordering::Relaxed);
    }

    pub fn wait_for_encoding(&self) -> Vec<u8> {
        // This is fine because the lock will stay in place until the recorder
        // is done. Therefore, we do not need to wait for it too.
        let mut locker = self.encoder.lock().unwrap();
        locker.take().unwrap().stop_consuming()
    }
}
