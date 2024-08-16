use crate::{region_selector::Region, statics::run_thread};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use xcap::Monitor;

use super::{gif_encoder::GIFEncoder, mp4_encoder::MP4Encoder};

struct XCaptureEnumerator {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    fps: u32,
    last_capture: std::time::Instant,
    display_connection: *mut std::ffi::c_void,
}

extern "C" {
    fn magiccap_recorder_x11_open_display() -> *mut std::ffi::c_void;
    fn magiccap_recorder_x11_close_display(display_connection: *mut std::ffi::c_void);
    fn magiccap_recorder_x11_get_region_rgba(
        display: *mut std::ffi::c_void,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        buf: *mut u8,
    ) -> bool;
}

impl XCaptureEnumerator {
    pub fn new(monitor: Monitor, region: Region, fps: u32) -> Self {
        let x = region.x + monitor.x();
        let y = region.y + monitor.y();
        Self {
            x,
            y,
            width: region.width,
            height: region.height,
            fps,
            last_capture: unsafe { std::mem::zeroed() },
            display_connection: unsafe { magiccap_recorder_x11_open_display() },
        }
    }

    pub fn next(&mut self) -> Option<Vec<u8>> {
        // Figure out how long to wait before capturing the next frame.
        let frame_duration = std::time::Duration::from_secs(1) / self.fps;
        let elapsed = self.last_capture.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }

        // Perform the capture.
        let mut buf = Vec::with_capacity((self.width * self.height * 4) as usize);
        unsafe { buf.set_len((self.width * self.height * 4) as usize) };
        let frame_ok = unsafe {
            magiccap_recorder_x11_get_region_rgba(
                self.display_connection,
                self.x,
                self.y,
                self.width,
                self.height,
                buf.as_mut_ptr(),
            )
        };
        if !frame_ok {
            return None;
        }

        // Update the last capture time.
        self.last_capture = std::time::Instant::now();

        // Return the frame.
        Some(buf)
    }
}

impl Drop for XCaptureEnumerator {
    fn drop(&mut self) {
        unsafe { magiccap_recorder_x11_close_display(self.display_connection) };
    }
}

struct PipewireCaptureEnumerator {}

impl PipewireCaptureEnumerator {
    pub fn new(monitor: Monitor, region: Region, fps: u32) -> Self {
        // TODO
        Self {}
    }

    pub fn next(&mut self) -> Option<Vec<u8>> {
        // TODO
        None
    }
}

struct CaptureEnumerator {
    x: Option<XCaptureEnumerator>,
    pipewire: Option<PipewireCaptureEnumerator>,
}

impl CaptureEnumerator {
    pub fn new(monitor: Monitor, region: Region, fps: u32) -> Self {
        // Determine which enumerator to use from the environment.
        if std::env::var("XDG_SESSION_TYPE").unwrap() == "wayland" {
            Self {
                x: None,
                pipewire: Some(PipewireCaptureEnumerator::new(monitor, region, fps)),
            }
        } else {
            Self {
                x: Some(XCaptureEnumerator::new(monitor, region, fps)),
                pipewire: None,
            }
        }
    }

    pub fn next(&mut self) -> Option<Vec<u8>> {
        if let Some(x) = &mut self.x {
            x.next()
        } else if let Some(pipewire) = &mut self.pipewire {
            pipewire.next()
        } else {
            None
        }
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

// Defines the structure that holds together all of the multi-threaded parts
// of the Linux MP4 recording logic.
pub struct PlatformSpecificMP4Recorder {
    abort: Arc<AtomicBool>,
    encoder: Arc<Mutex<Option<MP4Encoder>>>,
    ui: Arc<Mutex<Option<UIController>>>,
}

impl PlatformSpecificMP4Recorder {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        // Create the structure and all of the Arc's.
        let encoder_arc = Arc::new(Mutex::new(Some(MP4Encoder::new(
            region.width,
            region.height,
            30,
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
            let mut e = CaptureEnumerator::new(monitor, region, 30);
            loop {
                if atom_arc_clone.load(Ordering::Relaxed) {
                    return;
                }
                match e.next() {
                    Some(v) => guarded_value.consume_rgba_frame(v),
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

    pub fn wait_for_stop(&self) {
        let v = self.encoder.lock().unwrap();
        drop(v);
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
            let mut e = CaptureEnumerator::new(monitor, region, 15);
            loop {
                if atom_arc_clone.load(Ordering::Relaxed) {
                    return;
                }
                match e.next() {
                    Some(v) => guarded_value.consume_rgba_frame(v),
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

    pub fn wait_for_stop(&self) {
        let v = self.encoder.lock().unwrap();
        drop(v);
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
