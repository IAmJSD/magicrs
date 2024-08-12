use crate::statics::run_thread;
use std::{
    collections::HashMap,
    sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender},
};

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
    last: usize,
}

impl<V> Deque<V> {
    pub fn new() -> Self {
        Self {
            first: None,
            last: 0,
        }
    }

    pub fn to_stack(self) -> Option<Box<DequeItem<V>>> {
        self.first
    }

    pub fn push_end(&mut self, value: V) {
        if self.first.is_none() {
            let deque_box = Box::into_raw(Box::new(DequeItem {
                next: None,
                value,
            }));
            self.first.replace(unsafe { Box::from_raw(deque_box) });
            self.last = deque_box as usize;
            return;
        }

        let last_item = self.last as *mut DequeItem<V>;
        let new_item = Box::into_raw(Box::new(DequeItem {
            next: None,
            value,
        }));
        unsafe {
            (*last_item).next.replace(Box::from_raw(new_item));
        }
        self.last = new_item as usize;
    }
}

fn encode_worker(
    w: u32,
    h: u32,
    fps: u32,
    rgba_out: Receiver<RGBAInput>,
    gif_in: SyncSender<Vec<u8>>,
) {
    // Get the sleep time for 2 frames.
    let sleep_time = 1000 / fps * 2;

    // Defines a mapping of colors to their frequency.
    let mut color_map: HashMap<u32, u32> = HashMap::new();

    // Go through each frame as they arrive.
    let mut frame_dq = Deque::new();
    loop {
        let next_potential_frame = rgba_out.recv().unwrap();
        if let RGBAInput::Data(frame) = next_potential_frame {
            // Validate the image is divisible by 4. If not, continue.
            if frame.len() % 4 != 0 {
                continue;
            }

            // Iterate over the image.
            for i in (0..frame.len()).step_by(4) {
                // Get the color.
                let color = (frame[i] as u32) << 24
                    | (frame[i + 1] as u32) << 16
                    | (frame[i + 2] as u32) << 8
                    | (frame[i + 3] as u32);

                // Check if the color is in the map.
                if color_map.contains_key(&color) {
                    // Increment the value.
                    let value = color_map.get_mut(&color).unwrap();
                    *value += 1;
                } else {
                    // Insert the value.
                    color_map.insert(color, 1);
                }
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

        // Sleep for 2 frames so we do not burn aa CPU core.
        std::thread::sleep(std::time::Duration::from_millis(sleep_time as u64));
    }

    // Create the vector for the colors.
    let map_len = color_map.len();
    let mut vec_color_cap = map_len;
    if vec_color_cap > 256 {
        vec_color_cap = 256;
    }
    let mut vec: Vec<u8> = Vec::with_capacity(vec_color_cap * 4);

    // If the map lengtRGBAInput::Da is <= 256, just plop the map into this.
    if map_len <= 256 {
        // Iterate over the map.
        for (key, _) in color_map {
            // Push the key into the vec.
            vec.append(&mut vec![
                (key >> 24 & 0xFF) as u8,
                (key >> 16 & 0xFF) as u8,
                (key >> 8 & 0xFF) as u8,
                (key & 0xFF) as u8,
            ]);
        }
    } else {
        // Get the map sorted by value from highest to lowest.
        let mut sorted_map: Vec<(&u32, &u32)> = color_map.iter().collect();
        sorted_map.sort_by(|a, b| b.1.cmp(a.1));

        // Iterate over the map.
        for (key, _) in sorted_map {
            // Push the key into the vec.
            vec.append(&mut vec![
                (key >> 24 & 0xFF) as u8,
                (key >> 16 & 0xFF) as u8,
                (key >> 8 & 0xFF) as u8,
                (key & 0xFF) as u8,
            ]);

            // Check if we have enough colors.
            if vec.len() == 256*4 {
                break
            }
        }
    }

    // Create the gif encoder.
    let mut input = Vec::new();
    let mut encoder = gif::Encoder::new(
        &mut input,
        w as u16,
        h as u16,
        vec.as_slice(),
    ).unwrap();

    // Pass each frame to the encoder.
    let mut stack_val = frame_dq.to_stack();
    while let Some(mut frame) = stack_val {
        // Give the frame to the encoder.
        encoder.write_frame(
            &gif::Frame::from_rgba_speed(
                w as u16,
                h as u16,
                frame.value.as_mut(),
                1000 / fps as i32,
            )
        ).unwrap();

        // Set the next value.
        stack_val = frame.next;
    }

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
