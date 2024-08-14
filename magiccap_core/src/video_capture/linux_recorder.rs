use crate::{region_selector::Region, statics::run_thread};
use less_avc::{
    ycbcr_image::{Planes, YCbCrImage},
    H264Writer,
};
use mp4::{AacConfig, AvcConfig, TrackConfig, TrackType};
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, sync_channel, Receiver, Sender, SyncSender},
        Arc, Mutex,
    },
};
use xcap::Monitor;

use super::gif_encoder::GIFEncoder;

struct CaptureData {
    frame: Vec<u8>,
    pcm_audio: Option<Vec<u8>>,
}

struct CaptureEnumerator {}

impl CaptureEnumerator {
    pub fn new(monitor: Monitor, region: Region, fps: u32, audio: bool) -> Self {
        // TODO
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

// Code is from https://github.com/image-rs/image/blob/e176cd414ac6cc73909ff162e5d0b677f9d4fb08/src/codecs/jpeg/encoder.rs#L773
// It is private so I had to copy it out.
#[inline]
fn rgb_to_ycbcr(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let max: f32 = u8::MAX.to_f32().unwrap();
    let r: f32 = r.to_f32().unwrap();
    let g: f32 = g.to_f32().unwrap();
    let b: f32 = b.to_f32().unwrap();

    // Coefficients from JPEG File Interchange Format (Version 1.02), multiplied for 255 maximum.
    let y = 76.245 / max * r + 149.685 / max * g + 29.07 / max * b;
    let cb = -43.0185 / max * r - 84.4815 / max * g + 127.5 / max * b + 128.;
    let cr = 127.5 / max * r - 106.7685 / max * g - 20.7315 / max * b + 128.;

    (y as u8, cb as u8, cr as u8)
}

fn mp4_process_data(
    w: u32,
    h: u32,
    fps: u32,
    avc_encoder: &mut H264Writer<Vec<u8>>,
    data: CaptureData,
) {
    // Write the audio if present.
    if let Some(pcm_audio) = data.pcm_audio {
        // TODO: Add audio data
    }

    let chan_size = data.frame.len() / 4;
    let mut y = Vec::with_capacity(chan_size);
    let mut cb = Vec::with_capacity(chan_size);
    let mut cr = Vec::with_capacity(chan_size);
    for s in data.frame.chunks_exact(4) {
        let r = unsafe { s.get_unchecked(0) };
        let g = unsafe { s.get_unchecked(1) };
        let b = unsafe { s.get_unchecked(2) };
        let (y_v, cb_v, cr_v) = rgb_to_ycbcr(*r, *g, *b);
        y.push(y_v);
        cb.push(cb_v);
        cr.push(cr_v);
    }

    // Write the YCbCr image to the AVC encoder.
    avc_encoder
        .write(&YCbCrImage {
            planes: Planes(),
            height: h,
            width: w,
        })
        .unwrap();
}

enum CaptureDataInput {
    Data(CaptureData),
    Encode,
    Abort,
}

fn mp4_encode_worker(
    w: u32,
    h: u32,
    fps: u32,
    data_out: Receiver<CaptureDataInput>,
    mp4_in: SyncSender<Vec<u8>>,
) {
    // Build the AVC and AAC encoders.
    let mut avc_vec = Vec::new();
    let mut avc_enc = less_avc::H264Writer::new(&mut avc_vec).unwrap();

    // Handle all of the incoming data.
    loop {
        let next_potential_data = data_out.recv().unwrap();
        if let CaptureDataInput::Data(data) = next_potential_data {
            // Write the data.
            mp4_write_data(w, h, fps, &mut mp4_writer, data);
        } else {
            // If abort, return now. Otherwise, we should break.
            if let CaptureDataInput::Abort = next_potential_data {
                return;
            } else {
                break;
            }
        }
    }

    // Build the MP4 writer.
    let data = Cursor::new(Vec::<u8>::new());
    let config = mp4::Mp4Config {
        major_brand: str::parse("isom").unwrap(),
        minor_version: 512,
        compatible_brands: vec![
            str::parse("isom").unwrap(),
            str::parse("iso2").unwrap(),
            str::parse("avc1").unwrap(),
            str::parse("mp41").unwrap(),
        ],
        timescale: 1000,
    };
    let mut mp4_writer = mp4::Mp4Writer::write_start(data, &config).unwrap();

    // Write tracks 0 and 1.
    mp4_writer.add_track(&TrackConfig {
        track_type: TrackType::Video,
        timescale: 1000,
        language: "eng".to_string(),
        media_conf: mp4::MediaConfig::AvcConfig(AvcConfig {
            width: w as u16,
            height: h as u16,
            // TODO
            seq_param_set,
            pic_param_set,
        }),
    });
    mp4_writer.add_track(&TrackConfig {
        track_type: TrackType::Audio,
        timescale: 1000,
        language: "eng".to_string(),
        media_conf: mp4::MediaConfig::AacConfig(AacConfig {
            bitrate: 32000,
            profile: mp4::AudioObjectType::AacMain,
            freq_index: mp4::SampleFreqIndex::Freq48000,
            chan_conf: mp4::ChannelConfig::Stereo,
        }),
    });

    // Write the encoded data.
    mp4_writer.write_end().unwrap();
    mp4_in.send(mp4_writer.into_writer().into_inner()).unwrap()
}

struct MP4Encoder {
    data_in: Sender<CaptureDataInput>,
    mp4_out: Option<Receiver<Vec<u8>>>,
}

impl MP4Encoder {
    pub fn new(w: u32, h: u32, fps: u32) -> Self {
        let (data_in, data_out) = channel();
        let (mp4_in, mp4_out) = sync_channel(0);
        run_thread(move || mp4_encode_worker(w, h, fps, data_out, mp4_in));
        Self {
            data_in,
            mp4_out: Some(mp4_out),
        }
    }

    pub fn consume_data(&self, data: CaptureData) {
        self.data_in.send(CaptureDataInput::Data(data)).unwrap()
    }

    pub fn stop_consuming(mut self) -> Vec<u8> {
        let out_chan = match self.mp4_out.take() {
            Some(v) => v,
            None => panic!("stop encoding was called twice!"),
        };
        self.data_in.send(CaptureDataInput::Encode).unwrap();
        out_chan.recv().unwrap()
    }
}

impl Drop for MP4Encoder {
    fn drop(&mut self) {
        if self.mp4_out.is_some() {
            self.data_in.send(CaptureDataInput::Abort).unwrap();
        }
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
            let mut e = CaptureEnumerator::new(monitor, region, 30, true);
            loop {
                if atom_arc_clone.load(Ordering::Relaxed) {
                    return;
                }
                match e.next() {
                    Some(v) => guarded_value.consume_data(v),
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
