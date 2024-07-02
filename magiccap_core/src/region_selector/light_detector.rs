use image::RgbaImage;

pub struct LightDetector {
    image: RgbaImage,
    cached: Vec<Option<bool>>,
}

static DEFAULT_CHUNK_COUNT: u32 = 250;

impl LightDetector {
    // Creates a new light detector with the given image.
    pub fn new(image: RgbaImage) -> Self {
        Self {
            image,
            cached: vec![None; DEFAULT_CHUNK_COUNT as usize],
        }
    }

    // Gets the lightness of the chunk the cursor is on.
    pub fn get_lightness(&mut self, cursor_x: u32, cursor_y: u32) -> bool {
        let (img_width, img_height) = self.image.dimensions();

        // Calculate the number of chunks per row and column.
        let chunks_per_side = (DEFAULT_CHUNK_COUNT as f64).sqrt() as u32;

        // Calculate the chunk width and height.
        let chunk_w = img_width / chunks_per_side;
        let chunk_h = img_height / chunks_per_side;

        // Ensure the cursor position is within image bounds.
        let cursor_x = cursor_x.min(img_width - 1);
        let cursor_y = cursor_y.min(img_height - 1);

        // Determine the chunk index based on the cursor position.
        let chunk_x = cursor_x / chunk_w;
        let chunk_y = cursor_y / chunk_h;
        let mut chunk_index = (chunk_y * chunks_per_side + chunk_x) as usize;
        if chunk_index >= self.cached.len() {
            // Hmm, weird. Not worth panicking over.
            chunk_index = self.cached.len() - 1;
        }

        // Check if we have a cached result for this chunk.
        if let Some(cached_result) = self.cached[chunk_index] {
            return cached_result;
        }

        // Calculate the average brightness for the chunk.
        let mut total_brightness = 0u64;
        let mut pixel_count = 0u64;

        let start_x = chunk_x * chunk_w;
        let start_y = chunk_y * chunk_h;
        let mut end_x = (chunk_x + 1) * chunk_w;
        let mut end_y = (chunk_y + 1) * chunk_h;
        if end_x >= img_width {
            end_x = img_width - 1;
        }
        if end_y >= img_height {
            end_y = img_height - 1;
        }

        for y in start_y..end_y {
            for x in start_x..end_x {
                let pixel = self.image.get_pixel(x, y);
                let brightness = (pixel.0[0] as u64 + pixel.0[1] as u64 + pixel.0[2] as u64) / 3;
                total_brightness += brightness;
                pixel_count += 1;
            }
        }

        let average_brightness = total_brightness / if pixel_count == 0 { 1 } else { pixel_count };
        let is_light = average_brightness > 128; // Threshold for light/dark

        // Cache the result.
        self.cached[chunk_index] = Some(is_light);

        is_light
    }
}
