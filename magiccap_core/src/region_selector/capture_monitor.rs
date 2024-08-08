use image::RgbaImage;
use xcap::{Monitor, XCapResult};

// Draws the OS cursor on Linux.
#[cfg(target_os = "linux")]
fn draw_cursor(img: &mut RgbaImage, x: i32, y: i32) {
    use image::DynamicImage;

    // Get the cursor as a image view.
    enum RGBOrRGBA {
        Rgb(image::RgbImage),
        Rgba(image::RgbaImage),
    }
    let view = crate::mainthread::main_thread_sync(|| {
        // Get the pixbuf for the cursor.
        let cursor =
            gdk::Cursor::for_display(&gdk::Display::default().unwrap(), gdk::CursorType::LeftPtr)
                .unwrap();
        let pixbuf = cursor.image().unwrap();

        // Get the width/height/channels/pixels for this cursor.
        let width = pixbuf.width() as u32;
        let height = pixbuf.height() as u32;
        let num_channels = pixbuf.n_channels();
        let pixels = unsafe { pixbuf.pixels() }.to_vec();

        // 4 channels is most likely, so handle that first.
        if num_channels == 4 {
            return RGBOrRGBA::Rgba(image::RgbaImage::from_vec(width, height, pixels).unwrap());
        }

        // Handles the RGB case.
        RGBOrRGBA::Rgb(image::RgbImage::from_vec(width, height, pixels).unwrap())
    });

    // Draw onto the image at the specified co-ordinate.
    match view {
        RGBOrRGBA::Rgb(rgb_cursor) => {
            image::imageops::overlay(img, &DynamicImage::from(rgb_cursor), x as i64, y as i64);
        }
        RGBOrRGBA::Rgba(rgba_cursor) => {
            image::imageops::overlay(img, &rgba_cursor, x as i64, y as i64);
        }
    }
}

// Does the monitor capture including the cursor if specified.
pub fn capture_monitor(monitor: &Monitor, mouse_pos: Option<(i32, i32)>) -> XCapResult<RgbaImage> {
    // Attempt the capture.
    let mut rgba = match monitor.capture_image() {
        Ok(i) => i,
        Err(e) => return XCapResult::Err(e),
    };

    #[cfg(not(target_os = "windows"))]
    if let Some((mouse_x, mouse_y)) = mouse_pos {
        // Check if it is on this monitor.
        let monitor_x = monitor.x();
        let monitor_y = monitor.y();
        let monitor_w = monitor.width() as i32;
        let monitor_h = monitor.height() as i32;
        if mouse_x >= monitor_x
            && monitor_x + monitor_w >= mouse_x
            && mouse_y >= monitor_y
            && monitor_y + monitor_h >= mouse_y
        {
            // We are. Get the cursor and draw it.
            draw_cursor(&mut rgba, mouse_x - monitor_x, mouse_y - monitor_y)
        }
    }

    // Return the data as okay.
    XCapResult::Ok(rgba)
}
