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
    fn magiccap_recorder_x11_close_display(display: *mut std::ffi::c_void);
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

    pub fn next(&mut self, v: &mut Vec<u8>) -> bool {
        // Figure out how long to wait before capturing the next frame.
        let frame_duration = std::time::Duration::from_secs(1) / self.fps;
        let elapsed = self.last_capture.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }

        // Perform the capture.
        let frame_ok = unsafe {
            magiccap_recorder_x11_get_region_rgba(
                self.display_connection,
                self.x,
                self.y,
                self.width,
                self.height,
                v.as_mut_ptr(),
            )
        };
        if !frame_ok {
            return false;
        }

        // Update the last capture time.
        self.last_capture = std::time::Instant::now();

        // Return ok.
        true
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

    pub fn next(&mut self, v: &mut Vec<u8>) -> bool {
        // TODO
        false
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

    pub fn next(&mut self, v: &mut Vec<u8>) -> bool {
        if let Some(x) = &mut self.x {
            x.next(v)
        } else if let Some(pipewire) = &mut self.pipewire {
            pipewire.next(v)
        } else {
            false
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
pub struct PlatformSpecificMP4Recorder<'a> {
    abort: Arc<AtomicBool>,
    encoder: FakeSend<*mut Mutex<Option<MP4Encoder<'a>>>>,
    ui: Arc<Mutex<Option<UIController>>>,
}

struct FakeSend<T>(T);
unsafe impl<T> Send for FakeSend<T> {}
unsafe impl<T> Sync for FakeSend<T> {}

impl<'a> PlatformSpecificMP4Recorder<'_> {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        // Create the structure and all of the Arc's. This is okay to do because we will always
        // be the last person holding the encoder.
        let encoder_raw = Box::into_raw(Box::new(Mutex::new(Some(MP4Encoder::new(
            region.width,
            region.height,
            30,
        )))));
        let encoder_raw_cpy = encoder_raw as usize;
        let atom_arc = Arc::new(AtomicBool::new(false));
        let atom_arc_clone = Arc::clone(&atom_arc);
        let ui_arc = Arc::new(Mutex::new(Some(UIController::new(
            monitor.clone(),
            region.clone(),
        ))));
        let ui_arc_clone = Arc::clone(&ui_arc);
        let v = Self {
            abort: atom_arc,
            encoder: FakeSend(encoder_raw),
            ui: ui_arc,
        };
        let w = region.width;
        let h = region.height;

        // Create the recording thread.
        let mut buf = Vec::with_capacity((w * h * 4) as usize);
        unsafe {
            buf.set_len(buf.capacity());
        }
        run_thread(move || {
            let encoder_raw = encoder_raw_cpy as *mut Mutex<Option<MP4Encoder>>;
            let mut lock = unsafe { &*encoder_raw }.lock().unwrap();
            let guarded_value = lock.as_mut().unwrap();
            let mut e = CaptureEnumerator::new(monitor, region, 15);
            let buf_mut = buf.as_mut();
            loop {
                if atom_arc_clone.load(Ordering::Relaxed) {
                    return;
                }
                if !e.next(buf_mut) {
                    return;
                }
                let buf_mut_cpy = unsafe { &mut *(buf_mut as *mut _) };
                guarded_value.consume_rgba_frame(buf_mut_cpy);
                if let Some(ui_controller) = ui_arc_clone.lock().unwrap().as_mut() {
                    ui_controller.new_frame();
                }
            }
        });

        // Return the structure for usage by the higher level APIs.
        v
    }

    pub fn wait_for_stop(&self) {
        let v = unsafe { &*self.encoder.0 }.lock().unwrap();
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
        let mut locker = unsafe { &*self.encoder.0 }.lock().unwrap();
        locker.take().unwrap().stop_consuming()
    }
}

impl Drop for PlatformSpecificMP4Recorder<'_> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.encoder.0));
        }
    }
}

// Defines the structure that holds together all of the multi-threaded parts
// of the Linux GIF recording logic.
pub struct PlatformSpecificGIFRecorder<'a> {
    abort: Arc<AtomicBool>,
    encoder: FakeSend<*mut Mutex<Option<GIFEncoder<'a>>>>,
    ui: Arc<Mutex<Option<UIController>>>,
}

impl<'a> PlatformSpecificGIFRecorder<'_> {
    pub fn new(monitor: Monitor, region: Region) -> Self {
        // Create the structure and all of the Arc's. This is okay to do because we will always
        // be the last person holding the encoder.
        let encoder_raw = Box::into_raw(Box::new(Mutex::new(Some(GIFEncoder::new(
            region.width,
            region.height,
            15,
        )))));
        let encoder_raw_cpy = encoder_raw as usize;
        let atom_arc = Arc::new(AtomicBool::new(false));
        let atom_arc_clone = Arc::clone(&atom_arc);
        let ui_arc = Arc::new(Mutex::new(Some(UIController::new(
            monitor.clone(),
            region.clone(),
        ))));
        let ui_arc_clone = Arc::clone(&ui_arc);
        let v = Self {
            abort: atom_arc,
            encoder: FakeSend(encoder_raw),
            ui: ui_arc,
        };
        let w = region.width;
        let h = region.height;

        // Create the recording thread.
        let mut buf = Vec::with_capacity((w * h * 4) as usize);
        unsafe {
            buf.set_len(buf.capacity());
        }
        run_thread(move || {
            let encoder_raw = encoder_raw_cpy as *mut Mutex<Option<GIFEncoder>>;
            let mut lock = unsafe { &*encoder_raw }.lock().unwrap();
            let guarded_value = lock.as_mut().unwrap();
            let mut e = CaptureEnumerator::new(monitor, region, 15);
            let buf_mut = buf.as_mut();
            loop {
                if atom_arc_clone.load(Ordering::Relaxed) {
                    return;
                }
                if !e.next(buf_mut) {
                    return;
                }
                let buf_mut_cpy = unsafe { &mut *(buf_mut as *mut _) };
                guarded_value.consume_rgba_frame(buf_mut_cpy);
                if let Some(ui_controller) = ui_arc_clone.lock().unwrap().as_mut() {
                    ui_controller.new_frame();
                }
            }
        });

        // Return the structure for usage by the higher level APIs.
        v
    }

    pub fn wait_for_stop(&self) {
        let v = unsafe { &*self.encoder.0 }.lock().unwrap();
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
        let mut locker = unsafe { &*self.encoder.0 }.lock().unwrap();
        locker.take().unwrap().stop_consuming()
    }
}

impl Drop for PlatformSpecificGIFRecorder<'_> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.encoder.0));
        }
    }
}
