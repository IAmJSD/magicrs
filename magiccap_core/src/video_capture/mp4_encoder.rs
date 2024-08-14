use crate::statics::run_thread;
use less_avc::{
    ycbcr_image::{DataPlane, Planes, YCbCrImage},
    BitDepth, H264Writer,
};
use mp4::{AvcConfig, Bytes, Mp4Sample, TrackConfig, TrackType};
use std::{
    io::Cursor,
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

fn mp4_process_data(w: u32, h: u32, avc_encoder: &mut H264Writer<&mut Vec<u8>>, data: Vec<u8>) {
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

enum CaptureDataInput {
    Data(Vec<u8>),
    Encode,
    Abort,
}

fn mp4_encode_worker(
    w: u32,
    h: u32,
    data_out: Receiver<CaptureDataInput>,
    mp4_in: SyncSender<Vec<u8>>,
) {
    // Build the AVC encoder.
    let mut avc_vec = Vec::new();
    let mut start_time = 0;
    let mut avc_enc = less_avc::H264Writer::new(&mut avc_vec).unwrap();

    // Handle all of the incoming data.
    loop {
        let next_potential_data = data_out.recv().unwrap();
        if let CaptureDataInput::Data(data) = next_potential_data {
            // If start time is 0, set it now.
            if start_time == 0 {
                start_time = std::time::Instant::now().elapsed().as_millis() as u32;
            }

            // Write the data.
            mp4_process_data(w, h, &mut avc_enc, data);
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
    let seq_param_set = vec![
        0x67, 0x42, 0x00, 0x0A, 0xFF, 0xE1, 0x00, 0x1F, 0x27, 0x42, 0x00, 0x0A, 0x9A, 0x00, 0x00,
        0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80,
    ];
    let pic_param_set = vec![0x68, 0xCE, 0x3C, 0x80];
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

    // Write the AVC data.
    let _ = mp4_writer.write_sample(
        0,
        &Mp4Sample {
            start_time: 0,
            duration: std::time::Instant::now().elapsed().as_millis() as u32 - start_time,
            rendering_offset: 0,
            is_sync: true,
            bytes: Bytes::copy_from_slice(avc_vec.as_slice()),
        },
    );

    // Send the encoded data.
    mp4_writer.write_end().unwrap();
    mp4_in.send(mp4_writer.into_writer().into_inner()).unwrap()
}

pub struct MP4Encoder {
    data_in: Sender<CaptureDataInput>,
    mp4_out: Option<Receiver<Vec<u8>>>,
}

impl MP4Encoder {
    pub fn new(w: u32, h: u32, fps: u32) -> Self {
        let (data_in, data_out) = channel();
        let (mp4_in, mp4_out) = sync_channel(0);
        run_thread(move || mp4_encode_worker(w, h, data_out, mp4_in));
        Self {
            data_in,
            mp4_out: Some(mp4_out),
        }
    }

    pub fn consume_rgba_frame(&self, frame: Vec<u8>) {
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

impl Drop for MP4Encoder {
    fn drop(&mut self) {
        if self.mp4_out.is_some() {
            self.data_in.send(CaptureDataInput::Abort).unwrap();
        }
    }
}
