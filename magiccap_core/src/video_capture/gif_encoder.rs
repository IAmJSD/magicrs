use crate::statics::run_thread;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};

struct PaletteGeneration {
    mapping: Vec<Vec<[usize; 256]>>,
    palette: Vec<u32>,
}

impl PaletteGeneration {
    pub fn new() -> Self {
        Self {
            mapping: vec![vec![[0; 256]; 256]; 256],
            palette: vec![0; 256],
        }
    }

    pub fn write_color_usage(&mut self, r: u8, g: u8, b: u8) {
        let mem_ptr = unsafe {
            self.mapping
                .get_unchecked_mut(r as usize)
                .get_unchecked_mut(g as usize)
                .get_unchecked_mut(b as usize)
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
                    .get_unchecked(r as usize)
                    .get_unchecked(g as usize)
                    .get_unchecked(b as usize)
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

enum RGBAInput {
    Data(Vec<u8>),
    Encode,
    Abort,
}

pub struct GIFEncoder {
    rgba_in: Sender<RGBAInput>,
    gif_out: Option<Receiver<Vec<u8>>>,
}

struct DequeItem<V> {
    next: Option<Box<DequeItem<V>>>,
    value: V,
}

struct Deque<V> {
    first: Option<Box<DequeItem<V>>>,
    last: *mut DequeItem<V>,
}

impl<V> Deque<V> {
    pub fn new() -> Self {
        Self {
            first: None,
            last: unsafe { std::mem::zeroed() },
        }
    }

    pub fn to_queue(self) -> Option<Box<DequeItem<V>>> {
        self.first
    }

    pub fn push_end(&mut self, value: V) {
        if self.first.is_none() {
            let mut deque_box = Box::new(DequeItem { next: None, value });
            self.last = deque_box.as_mut() as *mut _;
            self.first.replace(deque_box);
            return;
        }

        let mut new_item = Box::new(DequeItem { next: None, value });
        let curr_last = self.last;
        self.last = new_item.as_mut() as *mut _;
        unsafe {
            (*curr_last).next.replace(new_item);
        }
    }
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
    let mut frame_dq = Deque::new();
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
            frame_dq.push_end(frame);
        } else {
            // If abort, return now. Otherwise, we should break.
            if let RGBAInput::Abort = next_potential_frame {
                return;
            } else {
                break;
            }
        }
    }

    // Create the gif encoder.
    let mut input = Vec::new();
    let mut encoder =
        gif::Encoder::new(&mut input, w as u16, h as u16, &color_map.get_gif_palette()).unwrap();

    // Pass each frame to the encoder.
    let mut q_val = frame_dq.to_queue();
    while let Some(frame) = q_val {
        // Give the frame to the encoder.
        let mut decompressed_slice = frame.value;
        encoder
            .write_frame(&gif::Frame::from_rgba_speed(
                w as u16,
                h as u16,
                decompressed_slice.as_mut_slice(),
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

        // Set the next value.
        q_val = frame.next;
    }

    // Write the loop.
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    // Drop the encoder.
    drop(encoder);

    // Send the data.
    gif_in.send(input).unwrap();
}

impl GIFEncoder {
    pub fn new(w: u32, h: u32, fps: u32) -> Self {
        let (rgba_in, rgba_out) = channel();
        let (gif_in, gif_out) = sync_channel(0);
        run_thread(move || encode_worker(w, h, fps, rgba_out, gif_in));
        Self {
            rgba_in,
            gif_out: Some(gif_out),
        }
    }

    pub fn consume_rgba_frame(&self, frame: Vec<u8>) {
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

impl Drop for GIFEncoder {
    fn drop(&mut self) {
        if self.gif_out.is_some() {
            self.rgba_in.send(RGBAInput::Abort).unwrap();
        }
    }
}
