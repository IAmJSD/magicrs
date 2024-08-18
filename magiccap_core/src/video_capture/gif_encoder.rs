use super::rgba_compressor::RGBACompressor;
use crate::statics::run_thread;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};

struct PaletteGeneration {
    mapping: Vec<u32>,
    palette: Vec<u32>,
}

impl PaletteGeneration {
    pub fn new() -> Self {
        Self {
            mapping: vec![0; 256 * 256 * 256],
            palette: vec![0; 256],
        }
    }

    pub fn write_color_usage(&mut self, r: u8, g: u8, b: u8) {
        let mem_ptr = unsafe {
            self.mapping
                .get_unchecked_mut(r as usize * 256 * 256 + g as usize * 256 + b as usize)
        };
        let v = *mem_ptr + 1;
        *mem_ptr = v;
        let mut potential_drop_victim = -1;
        let joined_together = (r as u32) << 16 | (g as u32) << 8 | b as u32;
        for (i, color) in self.palette.iter().enumerate() {
            if color == &joined_together {
                return;
            }
            if color == &0 {
                potential_drop_victim = i as i32;
                break;
            }
            let (r, g, b) = (color >> 16, (color >> 8) & 0xFF, color & 0xFF);
            let usages = unsafe {
                self.mapping
                    .get_unchecked(r as usize * 256 * 256 + g as usize * 256 + b as usize)
            };
            if *usages < v {
                potential_drop_victim = i as i32;
                break;
            }
        }
        if potential_drop_victim == -1 {
            return;
        }
        self.palette[potential_drop_victim as usize] = joined_together;
    }

    pub fn get_gif_palette(self) -> [u8; 256 * 3] {
        let mut palette = [0; 256 * 3];
        for (i, color) in self.palette.iter().enumerate() {
            palette[i * 3] = (*color >> 16) as u8;
            palette[i * 3 + 1] = (*color >> 8) as u8;
            palette[i * 3 + 2] = *color as u8;
        }
        palette
    }
}

enum RGBAInput<'a> {
    Data(&'a mut Vec<u8>),
    Encode,
    Abort(SyncSender<()>),
}

pub struct GIFEncoder<'a> {
    rgba_in: Sender<RGBAInput<'a>>,
    gif_out: Option<Receiver<Vec<u8>>>,
}

fn encode_worker(
    w: u32,
    h: u32,
    fps: u32,
    rgba_out: Receiver<RGBAInput>,
    gif_in: SyncSender<Vec<u8>>,
) {
    // Defines a mapping of colors to their frequency.
    let mut color_map = PaletteGeneration::new();

    // Go through each frame as they arrive.
    let mut frame_vec = Vec::with_capacity(200);
    let mut compressor = RGBACompressor::new();
    loop {
        let next_potential_frame = rgba_out.recv().unwrap();
        if let RGBAInput::Data(frame) = next_potential_frame {
            // Validate the image is divisible by 4. If not, continue.
            if frame.len() % 4 != 0 {
                continue;
            }

            // Iterate over the image and write the color usage.
            for i in frame.chunks_exact(4) {
                color_map.write_color_usage(i[0], i[1], i[2]);
            }

            // Push to the deque.
            frame_vec.push(compressor.compress(frame));
        } else {
            // If abort, return now. Otherwise, we should break.
            if let RGBAInput::Abort(s) = next_potential_frame {
                s.send(()).unwrap();
                return;
            }
            break;
        }
    }

    // Create the gif encoder.
    let mut input = Vec::new();
    let mut encoder =
        gif::Encoder::new(&mut input, w as u16, h as u16, &color_map.get_gif_palette()).unwrap();

    // Pass each frame to the encoder.
    let mut buffer = Vec::with_capacity(w as usize * h as usize * 4);
    for frame in frame_vec {
        // Give the frame to the encoder.
        frame.decompress_into_buffer(&mut buffer);
        encoder
            .write_frame(&gif::Frame::from_rgba_speed(
                w as u16,
                h as u16,
                buffer.as_mut_slice(),
                10,
            ))
            .unwrap();

        // Add a delay for the FPS.
        let delay = 1000 / fps / 10;
        encoder
            .write_extension(gif::ExtensionData::new_control_ext(
                delay as u16,
                gif::DisposalMethod::Keep,
                false,
                None,
            ))
            .unwrap();
    }

    // Write the loop.
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    // Drop the encoder.
    drop(encoder);

    // Send the data.
    gif_in.send(input).unwrap();
}

impl<'a> GIFEncoder<'a> {
    pub fn new(w: u32, h: u32, fps: u32) -> Self {
        let (rgba_in, rgba_out) = channel();
        let (gif_in, gif_out) = sync_channel(0);

        // This is okay to do because everything the lifetime of the thread is tied to the lifetime
        // of the GIFEncoder since both Encode and Abort are the main control signals to it and both kill
        // the thread.
        let static_rgba_out = unsafe {
            std::mem::transmute::<Receiver<RGBAInput<'a>>, Receiver<RGBAInput<'static>>>(rgba_out)
        };
        run_thread(move || encode_worker(w, h, fps, static_rgba_out, gif_in));

        Self {
            rgba_in,
            gif_out: Some(gif_out),
        }
    }

    pub fn consume_rgba_frame(&self, frame: &'a mut Vec<u8>) {
        self.rgba_in.send(RGBAInput::Data(frame)).unwrap()
    }

    pub fn stop_consuming(mut self) -> Vec<u8> {
        let out_chan = match self.gif_out.take() {
            Some(v) => v,
            None => panic!("stop encoding was called twice!"),
        };
        self.rgba_in.send(RGBAInput::Encode).unwrap();
        out_chan.recv().unwrap()
    }
}

impl<'a> Drop for GIFEncoder<'_> {
    fn drop(&mut self) {
        if self.gif_out.is_some() {
            // Send a sync sender and wait for it. This allows us to ensure the lifetimes
            // are okay because it won't be processing any more frames after this.
            let (s, r) = sync_channel(0);
            self.rgba_in.send(RGBAInput::Abort(s)).unwrap();
            r.recv().unwrap();
        }
    }
}
