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

struct NormalisedOutput {
    t: String,
    end: String,
}

impl PartialEq for NormalisedOutput {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.end == other.end
    }
}

fn normalise(s: Option<String>) -> Option<NormalisedOutput> {
    match s {
        Some(v) => {
            let dash_split = v.splitn(2, '-').collect::<Vec<&str>>();
            match dash_split[0] {
                "DisplayPort" => Some(NormalisedOutput {
                    t: "DP".to_string(),
                    end: dash_split[1].to_string(),
                }),
                s if s.starts_with("HDMI") => Some(NormalisedOutput {
                    t: "HDMI".to_string(),
                    end: dash_split[1].to_string(),
                }),
                s if s.starts_with("VGA") => Some(NormalisedOutput {
                    t: "VGA".to_string(),
                    end: dash_split[1].to_string(),
                }),
                s if s.starts_with("DVI") => Some(NormalisedOutput {
                    t: "DVI".to_string(),
                    end: dash_split[1].to_string(),
                }),
                s if s.starts_with("eDP") => Some(NormalisedOutput {
                    t: "eDP".to_string(),
                    end: dash_split[1].to_string(),
                }),
                s if s.starts_with("DP") => Some(NormalisedOutput {
                    t: "DP".to_string(),
                    end: dash_split[1].to_string(),
                }),
                other => Some(NormalisedOutput {
                    t: other.to_string(),
                    end: if dash_split.len() > 1 {
                        dash_split[1].to_string()
                    } else {
                        "".to_string()
                    },
                }),
            }
        },
        None => None,
    }
}

#[cfg(target_os = "linux")]
fn capture_with_framebufferd(monitor: &Monitor) -> XCapResult<RgbaImage> {
    // Get the monitor's identifier.
    let name = monitor.name().unwrap_or("".to_string());

    // Get all DRM devices from framebufferd.
    let devices = match crate::framebufferd::list_devices() {
        Some(d) => d,
        None => {
            return XCapResult::Err(xcap::XCapError::Error(
                "Failed to list framebufferd devices".to_string(),
            ));
        }
    };

    // Filter to only DRM devices with valid fb_id and file_path.
    let drm_devices: Vec<_> = devices
        .into_iter()
        .filter(|d| {
            d.is_drm && (normalise(d.x11_output.clone()) == normalise(Some(name.clone())) || normalise(d.connector_name.clone()) == normalise(Some(name.clone())))
        })
        .collect();

    if drm_devices.is_empty() {
        eprintln!("No matching DRM devices found for monitor: {}", name);
        return XCapResult::Err(xcap::XCapError::Error(
            "No DRM devices available".to_string(),
        ));
    }

    // Capture this device
    let device = &drm_devices[0];
    let fb_id = device.fb_id.unwrap();
    match crate::framebufferd::get_drm_rgba_capture(fb_id, device.file_path.as_ref().unwrap().clone(), false) {
        Some(rgba_result) => {
            let img = RgbaImage::from_raw(
                rgba_result.width,
                rgba_result.height,
                rgba_result.data,
            )
            .ok_or_else(|| {
                xcap::XCapError::Error("Failed to create image from raw RGBA data".to_string())
            })?;
            XCapResult::Ok(img)
        }
        None => XCapResult::Err(xcap::XCapError::Error(
            "Failed to capture framebufferd device".to_string(),
        )),
    }
}

// Does the monitor capture including the cursor if specified.
pub fn capture_monitor(monitor: &Monitor, mouse_pos: Option<(i32, i32)>) -> XCapResult<RgbaImage> {
    // Attempt the capture.
    #[cfg(not(target_os = "linux"))]
    let mut rgba = match monitor.capture_image() {
        Ok(i) => i,
        Err(e) => return XCapResult::Err(e),
    };
    #[cfg(target_os = "linux")]
    let mut rgba = match capture_with_framebufferd(monitor) {
        Ok(i) => i,
        Err(e) => return XCapResult::Err(e),
    };

    #[cfg(not(target_os = "windows"))]
    if let Some((mouse_x, mouse_y)) = mouse_pos {
        // Check if it is on this monitor.
        let monitor_x = monitor.x().unwrap();
        let monitor_y = monitor.y().unwrap();
        let monitor_w = monitor.width().unwrap() as i32;
        let monitor_h = monitor.height().unwrap() as i32;
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
