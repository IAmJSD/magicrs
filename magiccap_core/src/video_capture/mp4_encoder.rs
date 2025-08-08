use crate::statics::run_thread;
use less_avc::{
    ycbcr_image::{DataPlane, Planes, YCbCrImage},
    BitDepth, H264Writer,
};
use mp4::{AvcConfig, Bytes, Mp4Sample, TrackConfig, TrackType};
use std::{
    cell::RefCell,
    io::{Cursor, Write},
    rc::Rc,
    sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender},
};

// Code is from https://github.com/image-rs/image/blob/e176cd414ac6cc73909ff162e5d0b677f9d4fb08/src/codecs/jpeg/encoder.rs#L773
// It is private so I had to copy it out.
#[inline]
fn rgb_to_ycbcr(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let max = u8::MAX as f32;
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;

    // Coefficients from JPEG File Interchange Format (Version 1.02), multiplied for 255 maximum.
    let y = 76.245 / max * r + 149.685 / max * g + 29.07 / max * b;
    let cb = -43.0185 / max * r - 84.4815 / max * g + 127.5 / max * b + 128.;
    let cr = 127.5 / max * r - 106.7685 / max * g - 20.7315 / max * b + 128.;

    (y as u8, cb as u8, cr as u8)
}

// A writer that allows us to observe the encoded bytes while the H.264 encoder holds a mutable borrow.
struct SharedVecWriter {
    inner: Rc<RefCell<Vec<u8>>>,
}

impl SharedVecWriter {
    fn new() -> (Self, Rc<RefCell<Vec<u8>>>) {
        let rc = Rc::new(RefCell::new(Vec::new()));
        (Self { inner: Rc::clone(&rc) }, rc)
    }
}

impl Write for SharedVecWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn mp4_process_data(
    w: u32,
    h: u32,
    avc_encoder: &mut H264Writer<&mut SharedVecWriter>,
    data: &mut Vec<u8>,
) {
    // Convert the RGBA frame to YCbCr.
    let chan_size = data.len() / 4;
    let mut y = Vec::with_capacity(chan_size);
    let mut cb = Vec::with_capacity(chan_size);
    let mut cr = Vec::with_capacity(chan_size);
    for s in data.chunks_exact(4) {
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
            planes: Planes::YCbCr((
                DataPlane {
                    bit_depth: BitDepth::Depth8,
                    stride: w as usize,
                    data: y.as_slice(),
                },
                DataPlane {
                    bit_depth: BitDepth::Depth8,
                    stride: w as usize,
                    data: cb.as_slice(),
                },
                DataPlane {
                    bit_depth: BitDepth::Depth8,
                    stride: w as usize,
                    data: cr.as_slice(),
                },
            )),
            height: h,
            width: w,
        })
        .unwrap();
}

enum CaptureDataInput<'a> {
    Data(&'a mut Vec<u8>),
    Encode,
    Abort(SyncSender<()>),
}

fn find_start_codes(data: &[u8]) -> Vec<usize> {
    // Finds indices of Annex B start codes (both 3 and 4 byte variants)
    let mut idxs = Vec::new();
    let mut i = 0;
    while i + 3 < data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
            idxs.push(i);
            i += 3;
            continue;
        }
        if i + 4 < data.len() && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 0 && data[i + 3] == 1 {
            idxs.push(i);
            i += 4;
            continue;
        }
        i += 1;
    }
    idxs
}

fn remove_start_code_prefix_len(data: &[u8], pos: usize) -> usize {
    // Return the number of bytes to skip for the start code at position pos
    if pos + 3 < data.len() && data[pos] == 0 && data[pos + 1] == 0 && data[pos + 2] == 1 {
        3
    } else if pos + 4 < data.len()
        && data[pos] == 0
        && data[pos + 1] == 0
        && data[pos + 2] == 0
        && data[pos + 3] == 1
    {
        4
    } else {
        0
    }
}

fn extract_sps_pps_from_annex_b(data: &[u8]) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let starts = find_start_codes(data);
    let mut sps: Option<Vec<u8>> = None;
    let mut pps: Option<Vec<u8>> = None;
    for (i, &start) in starts.iter().enumerate() {
        let sc_len = remove_start_code_prefix_len(data, start);
        let nal_start = start + sc_len;
        let nal_end = if i + 1 < starts.len() {
            starts[i + 1]
        } else {
            data.len()
        };
        if nal_start >= nal_end || nal_end > data.len() {
            continue;
        }
        let nal = &data[nal_start..nal_end];
        if nal.is_empty() {
            continue;
        }
        let nal_unit_type = nal[0] & 0x1F;
        match nal_unit_type {
            7 => {
                if sps.is_none() {
                    sps = Some(nal.to_vec());
                }
            }
            8 => {
                if pps.is_none() {
                    pps = Some(nal.to_vec());
                }
            }
            _ => {}
        }
        if sps.is_some() && pps.is_some() {
            break;
        }
    }
    (sps, pps)
}

fn convert_annex_b_to_mp4_sample(data: &[u8]) -> (Vec<u8>, bool) {
    // Convert an Annex B sequence (possibly multiple NAL units) into MP4 length-prefixed format.
    // Also detect if the sample contains an IDR slice (NAL type 5).
    let starts = find_start_codes(data);
    let mut out = Vec::with_capacity(data.len());
    let mut is_sync = false;
    for (i, &start) in starts.iter().enumerate() {
        let sc_len = remove_start_code_prefix_len(data, start);
        let nal_start = start + sc_len;
        let nal_end = if i + 1 < starts.len() {
            starts[i + 1]
        } else {
            data.len()
        };
        if nal_start >= nal_end || nal_end > data.len() {
            continue;
        }
        let nal = &data[nal_start..nal_end];
        if nal.is_empty() {
            continue;
        }
        let nal_unit_type = nal[0] & 0x1F;
        if nal_unit_type == 5 {
            is_sync = true;
        }
        let len = nal.len() as u32;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(nal);
    }
    (out, is_sync)
}

fn mp4_encode_worker(
    w: u32,
    h: u32,
    fps: u32,
    data_out: Receiver<CaptureDataInput<'static>>,
    mp4_in: SyncSender<Vec<u8>>,
) {
    // Build the AVC encoder with a shared writer buffer we can observe.
    let (mut shared_writer, shared_vec) = SharedVecWriter::new();
    let mut avc_enc = less_avc::H264Writer::new(&mut shared_writer).unwrap();

    // For tracking sample boundaries per-frame.
    let mut frame_ranges: Vec<(usize, usize)> = Vec::new();

    // Handle all of the incoming data.
    loop {
        let next_potential_data = data_out.recv().unwrap();
        if let CaptureDataInput::Data(data) = next_potential_data {
            let start = shared_vec.borrow().len();
            mp4_process_data(w, h, &mut avc_enc, data);
            let end = shared_vec.borrow().len();
            if end > start {
                frame_ranges.push((start, end));
            }
        } else {
            // If abort, return now. Otherwise, we should break.
            if let CaptureDataInput::Abort(s) = next_potential_data {
                s.send(()).unwrap();
                return;
            }
            break;
        }
    }

    // Snapshot the encoded bitstream.
    let avc_vec = shared_vec.borrow().clone();

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

    // Extract SPS/PPS from the stream.
    let (sps_opt, pps_opt) = extract_sps_pps_from_annex_b(&avc_vec);
    let (seq_param_set, pic_param_set) = match (sps_opt, pps_opt) {
        (Some(sps), Some(pps)) => (sps, pps),
        _ => {
            // If we somehow did not get SPS/PPS, send empty MP4 to avoid crash.
            mp4_writer.write_end().unwrap();
            let _ = mp4_in.send(mp4_writer.into_writer().into_inner());
            return;
        }
    };

    // Add the H.264 video track.
    let _ = mp4_writer.add_track(&TrackConfig {
        track_type: TrackType::Video,
        timescale: 1000,
        language: "eng".to_string(),
        media_conf: mp4::MediaConfig::AvcConfig(AvcConfig {
            width: w as u16,
            height: h as u16,
            seq_param_set,
            pic_param_set,
        }),
    });

    // Write one MP4 sample per frame with proper timing.
    let timescale: u32 = 1000;
    let mut current_start_time: u32 = 0;
    let frame_duration: u32 = if fps == 0 { 0 } else { timescale / fps };

    for (i, (start, end)) in frame_ranges.into_iter().enumerate() {
        let frame_data = &avc_vec[start..end];
        let (sample_bytes, is_sync) = convert_annex_b_to_mp4_sample(frame_data);
        let start_time = current_start_time;

        // Use rounded durations to minimize drift; last sample extends to the end.
        let duration = if frame_duration == 0 {
            0
        } else if i == 0 {
            frame_duration
        } else {
            frame_duration
        };
        current_start_time = start_time.saturating_add(duration);

        let _ = mp4_writer.write_sample(
            0,
            &Mp4Sample {
                start_time: start_time as u64,
                duration,
                rendering_offset: 0,
                is_sync,
                bytes: Bytes::copy_from_slice(sample_bytes.as_slice()),
            },
        );
    }

    // Finalize and send the encoded data.
    mp4_writer.write_end().unwrap();
    mp4_in
        .send(mp4_writer.into_writer().into_inner())
        .unwrap()
}

pub struct MP4Encoder<'a> {
    data_in: Sender<CaptureDataInput<'a>>,
    mp4_out: Option<Receiver<Vec<u8>>>,
}

impl<'a> MP4Encoder<'a> {
    pub fn new(w: u32, h: u32, fps: u32) -> Self {
        let (data_in, data_out) = channel();
        let (mp4_in, mp4_out) = sync_channel(0);

        // This is okay to do because everything the lifetime of the thread is tied to the lifetime
        // of the MP4Encoder since both Encode and Abort are the main control signals to it and both kill
        // the thread.
        let static_rgba_out = unsafe {
            std::mem::transmute::<Receiver<CaptureDataInput<'a>>, Receiver<CaptureDataInput<'static>>>(
                data_out,
            )
        };
        run_thread(move || mp4_encode_worker(w, h, fps, static_rgba_out, mp4_in));

        Self {
            data_in,
            mp4_out: Some(mp4_out),
        }
    }

    pub fn consume_rgba_frame(&self, frame: &'a mut Vec<u8>) {
        self.data_in.send(CaptureDataInput::Data(frame)).unwrap()
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

impl<'a> Drop for MP4Encoder<'_> {
    fn drop(&mut self) {
        if self.mp4_out.is_some() {
            let (s, r) = sync_channel(0);
            self.data_in.send(CaptureDataInput::Abort(s)).unwrap();
            r.recv().unwrap();
        }
    }
}
