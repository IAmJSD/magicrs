use super::{
    color_box,
    editors::{create_editor_vec, Editor, EditorFactory},
    event_loop_handler::{region_selector_event_loop_handler, region_selector_io_event_sent},
    gl_abstractions::GLTexture,
    light_detector::LightDetector,
    texture_pack::TexturePack,
    ui_renderer::region_selector_render_ui,
    RegionCapture,
};
use crate::mainthread::{main_thread_async, main_thread_sync};
use glfw::{Context, Glfw, PWindow};
use image::RgbaImage;
use once_cell::unsync::Lazy;
use std::sync::{Arc, RwLock};

// A container that bypasses the Send and Sync traits.
pub struct SendSyncBypass<T> {
    data: T,
}

impl<T> SendSyncBypass<T> {
    // Creates a new container with the given data.
    fn new(data: T) -> SendSyncBypass<T> {
        SendSyncBypass { data }
    }

    // Gets a mutable reference to the data.
    pub fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
unsafe impl<T> Send for SendSyncBypass<T> {}
unsafe impl<T> Sync for SendSyncBypass<T> {}

// Defines the items required to setup the region selector.
pub struct RegionSelectorSetup {
    #[allow(dead_code)] // Only dead in cases where the feature is disabled.
    pub windows: Vec<xcap::Window>,
    pub monitors: Vec<xcap::Monitor>,
    pub show_editors: bool,
    pub show_magnifier: bool,
    pub default_color: (u8, u8, u8),
}

// Doesn't actually matter here.
unsafe impl Send for RegionSelectorSetup {}

// Defines a editors usage.
pub struct EditorUsage {
    pub editor: Box<dyn Editor>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub display_index: usize,
}

// Defines the editor resizer element that is being dragged.
#[derive(Clone, Copy)]
pub enum EditorResizerElement {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Centre,
}

// Defines the context passed around internally.
pub struct RegionSelectorContext {
    // Pass through the GLFW instance for the main thread.
    pub glfw: &'static mut Glfw,

    // Defines everything created during initialization.
    pub setup: Box<RegionSelectorSetup>,
    pub glfw_windows: Vec<PWindow>,
    pub image_dimensions: Vec<(u32, u32)>,
    pub gl_screenshots: Vec<GLTexture>,
    pub light_detectors: Vec<LightDetector>,
    pub gl_screenshots_darkened: Vec<GLTexture>,
    pub editors: Vec<Lazy<Box<dyn EditorFactory>>>,
    pub black_texture: GLTexture,
    pub white_texture: GLTexture,
    pub striped_tex_w: GLTexture,
    pub striped_tex_h: GLTexture,
    pub texture_pack: TexturePack,

    // Defines event driven items.
    pub color_selection: Arc<RwLock<(u8, u8, u8, GLTexture)>>,
    pub active_selection: Option<(usize, (i32, i32))>,
    pub active_editors: Vec<EditorUsage>,
    pub editor_index: Option<usize>,
    pub editor_dragged: Option<(usize, EditorResizerElement)>,
    pub result: Option<RegionCapture>,
}

impl Drop for RegionSelectorContext {
    fn drop(&mut self) {
        // For some reason, the windows don't seem to close on their own.
        // Force close them in the OS.
        for window in std::mem::take(&mut self.glfw_windows) {
            let window_ptr = window.window_ptr();
            if !window_ptr.is_null() {
                unsafe {
                    glfw::ffi::glfwDestroyWindow(window_ptr);
                    glfw::ffi::glfwPollEvents();
                }
            }

            // DO NOT DROP THIS! We have already destroyed it above.
            std::mem::forget(window);
        }
    }
}

// Get the line textures used within the UI.
fn get_black_white_and_striped_texture(size: u32) -> (GLTexture, GLTexture, GLTexture, GLTexture) {
    let data = vec![0; size as usize * 4];
    let black = RgbaImage::from_vec(size, 1, data).unwrap();
    let black_tex = GLTexture::from_rgba(&black);

    // Create the white texture.
    let mut data = black.into_vec();
    data.iter_mut().for_each(|v| *v = 255);
    let white = RgbaImage::from_vec(size, 1, data).unwrap();
    let white_tex = GLTexture::from_rgba(&white);

    // Create the striped texture.
    let mut striped = white;
    for (i, pixel) in striped.pixels_mut().enumerate() {
        if i % 2 == 0 {
            *pixel = image::Rgba([0, 0, 0, 255]);
        }
    }
    let striped_tex_w = GLTexture::from_rgba(&striped);

    // Swap the width and height.
    let striped = RgbaImage::from_vec(1, size, striped.into_vec()).unwrap();
    let striped_tex_h = GLTexture::from_rgba(&striped);

    // Return the textures.
    (black_tex, white_tex, striped_tex_w, striped_tex_h)
}

// Handles iterating or jumping right to a index.
pub fn iter_windows_or_jump(
    ctx: &mut RegionSelectorContext,
    index: Option<usize>,
    closure: &dyn Fn(&mut RegionSelectorContext, &mut glfw::Window, usize),
) {
    // Use unsafe to get the mutable reference. This is safe because we know that the context will outlive
    // the mutable reference.
    let ctx2 = unsafe { &mut *(&mut *ctx as *mut RegionSelectorContext) };

    // Handle if the index is set.
    if let Some(index) = index {
        // Get the window.
        let window = &mut ctx2.glfw_windows[index];

        // Call the closure with separate mutable references.
        closure(ctx, window, index);
        return;
    }

    // Iterate through the screenshots.
    for (i, window) in ctx2.glfw_windows.iter_mut().enumerate() {
        // Call the closure with separate mutable references.
        closure(ctx, window, i);
    }
}

// Sets up the region selector.
fn setup_region_selector(
    setup: Box<RegionSelectorSetup>,
    screenshots: &mut Vec<RgbaImage>,
    glfw: SendSyncBypass<&'static mut Glfw>,
) -> Option<Box<SendSyncBypass<RegionSelectorContext>>> {
    // Go through each monitor and create a window for it.
    let glfw = glfw.data;
    let mut glfw_windows: Vec<PWindow> = Vec::with_capacity(setup.monitors.len());
    let mut largest_w_or_h = 0;
    if !glfw.with_connected_monitors(|glfw, glfw_monitors| {
        #[allow(unused_variables)] // Only used on Linux.
        for (index, monitor) in setup.monitors.iter().enumerate() {
            // Find the matching glfw monitor.
            let glfw_monitor = glfw_monitors
                .iter()
                .find(|m| m.get_pos() == (monitor.x().unwrap(), monitor.y().unwrap()))
                .unwrap();

            // Set the window hints to control the version of OpenGL.
            glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
            glfw.window_hint(glfw::WindowHint::OpenGlProfile(
                glfw::OpenGlProfileHint::Core,
            ));
            glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

            // Handle IO events.
            glfw.window_hint(glfw::WindowHint::CenterCursor(false));
            glfw.window_hint(glfw::WindowHint::FocusOnShow(true));

            // Handle borderless windows on Windows.
            #[cfg(target_os = "windows")]
            {
                glfw.window_hint(glfw::WindowHint::Decorated(false));
                glfw.window_hint(glfw::WindowHint::Floating(true));
                glfw.window_hint(glfw::WindowHint::Maximized(true));
                glfw.window_hint(glfw::WindowHint::Resizable(false));
            }

            // Create the window.
            let window_mode = if cfg!(target_os = "windows") {
                glfw::WindowMode::Windowed
            } else {
                glfw::WindowMode::FullScreen(&glfw_monitor)
            };
            #[allow(unused_mut)] // Only used on Windows.
            let mut window = match if glfw_windows.is_empty() {
                glfw.create_window(
                    monitor.width().unwrap(),
                    monitor.height().unwrap(),
                    "Region Selector",
                    window_mode,
                )
            } else {
                glfw_windows[0].create_shared(
                    monitor.width().unwrap(),
                    monitor.height().unwrap(),
                    "Region Selector",
                    window_mode,
                )
            } {
                Some(t) => t.0,
                None => {
                    for window in &mut glfw_windows {
                        window.set_should_close(true);
                    }
                    return false;
                }
            };

            // On Windows, set the position of the window to the monitor.
            #[cfg(target_os = "windows")]
            {
                window.set_monitor(
                    glfw::WindowMode::Windowed,
                    monitor.x(),
                    monitor.y(),
                    monitor.width(),
                    monitor.height(),
                    None,
                );
            }

            // Handle window servers on Linux.
            #[cfg(target_os = "linux")]
            {
                let x_ptr = window.get_x11_window();
                if !x_ptr.is_null() {
                    extern "C" {
                        fn magiccap_handle_linux_x11(
                            x_window_ptr: *mut std::ffi::c_void,
                            last: bool,
                        );
                    }
                    unsafe {
                        magiccap_handle_linux_x11(x_ptr, index == setup.monitors.len() - 1);
                    }
                }
            }

            // Push the window.
            glfw_windows.push(window);

            // Set the largest width or height.
            largest_w_or_h = largest_w_or_h.max(monitor.width().unwrap()).max(monitor.height().unwrap());
        }

        // Return true since success.
        true
    }) {
        return None;
    }

    // If there is no windows, return None.
    if glfw_windows.is_empty() {
        return None;
    }

    // Load in the OpenGL functions.
    let first_window_ref = &mut glfw_windows[0];
    gl::load_with(|s| first_window_ref.get_proc_address(s) as *const _);

    // Get the image dimensions.
    let image_dimensions = screenshots
        .iter()
        .map(|img| img.dimensions())
        .collect::<Vec<_>>();

    // Turn the images into textures.
    let gl_screenshots = screenshots
        .iter()
        .map(|img| GLTexture::from_rgba(&img))
        .collect::<Vec<_>>();

    // Get the light detectors.
    let light_detectors = screenshots
        .iter()
        .map(|img| LightDetector::new(img.clone()))
        .collect::<Vec<_>>();

    // Turn the images into darkened textures by manipulating the underlying data.
    // This is quicker than compiling a shader on first load and since we are not
    // mutating it in OpenGL, it will be very fast to blit from the texture.
    let gl_screenshots_darkened = screenshots
        .iter_mut()
        .map(|img| {
            // Darken the image.
            super::image_manipulation_simd::set_brightness_half_simd(img);

            // Create the texture.
            GLTexture::from_rgba(&img)
        })
        .collect::<Vec<_>>();

    // Create the context.
    let (black_texture, white_texture, striped_tex_w, striped_tex_h) =
        get_black_white_and_striped_texture(largest_w_or_h);
    let (dr, dg, db) = setup.default_color;
    let mut context = RegionSelectorContext {
        setup,
        glfw,
        glfw_windows,
        image_dimensions,
        gl_screenshots,
        light_detectors,
        gl_screenshots_darkened,
        editors: create_editor_vec(),
        black_texture,
        white_texture,
        striped_tex_w,
        striped_tex_h,
        texture_pack: TexturePack::new(),

        color_selection: Arc::new(RwLock::new((
            dr,
            dg,
            db,
            color_box::render_texture(dr, dg, db),
        ))),
        active_selection: None,
        active_editors: Vec::new(),
        editor_index: None,
        editor_dragged: None,
        result: None,
    };

    // Render the UI.
    unsafe { region_selector_render_ui(&mut context, true, None) };

    // Box the context.
    let mut ctx_boxed = Box::new(SendSyncBypass::new(context));

    // Handle GLFW events. We do some quite dangerous stuff here, but its okay because we know where this is polled.
    iter_windows_or_jump(&mut ctx_boxed.data, None, &|ctx, window, current_index| {
        // Make this window the current context.
        window.make_current();

        // Set the swap interval to adaptive vsync if this isn't Windows.
        ctx.glfw.set_swap_interval(if cfg!(target_os = "windows") {
            glfw::SwapInterval::None
        } else {
            glfw::SwapInterval::Adaptive
        });

        // Handle the mouse button being pressed.
        let ctx2 = unsafe { &mut *(&mut *ctx as *mut RegionSelectorContext) };
        window.set_mouse_button_callback(move |window, button, action, mods| {
            // Wrap it in a glfw::WindowEvent::MouseButton.
            let event = glfw::WindowEvent::MouseButton(button, action, mods);

            // Handle the event.
            region_selector_io_event_sent(ctx2, event, current_index as i32, window);
        });

        // Handle the scroll wheel being scrolled.
        let ctx2 = unsafe { &mut *(&mut *ctx as *mut RegionSelectorContext) };
        window.set_scroll_callback(move |window, x, y| {
            // Wrap it in a glfw::WindowEvent::Scroll.
            let event = glfw::WindowEvent::Scroll(x, y);

            // Handle the event.
            region_selector_io_event_sent(ctx2, event, current_index as i32, window);
        });

        // Handle a key being pressed.
        let ctx2 = unsafe { &mut *(&mut *ctx as *mut RegionSelectorContext) };
        window.set_key_callback(move |window, key, sc, action, modifiers| {
            // Wrap it in a glfw::WindowEvent::Key.
            let event = glfw::WindowEvent::Key(key, sc, action, modifiers);

            // Handle the event.
            region_selector_io_event_sent(ctx2, event, current_index as i32, window);
        });

        // Handle the cursor being moved.
        let ctx2 = unsafe { &mut *(&mut *ctx as *mut RegionSelectorContext) };
        window.set_cursor_pos_callback(move |window, x, y| {
            // Wrap it in a glfw::WindowEvent::CursorPos.
            let event = glfw::WindowEvent::CursorPos(x, y);

            // Handle the event.
            region_selector_io_event_sent(ctx2, event, current_index as i32, window);
        });
    });

    // Return the context.
    Some(ctx_boxed)
}

// Make sure a item gets dropped on the main thread.
fn main_thread_drop<T>(item: T)
where
    T: Send + 'static,
{
    main_thread_async(move || drop(item));
}

static GLFW_SETUP: RwLock<Option<SendSyncBypass<*mut Glfw>>> = RwLock::new(None);

// Sets up the GLFW instance for the region selector.
pub fn setup_glfw_instance_for_region_selector() {
    main_thread_async(|| {
        let mut write_guard = GLFW_SETUP.write().unwrap();
        let glfw = Box::leak(Box::new(glfw::init(glfw::fail_on_errors).unwrap()));
        *write_guard = Some(SendSyncBypass::new(&mut *glfw as *mut Glfw));
    });
}

// Invokes the engine.
pub fn invoke(
    setup: Box<RegionSelectorSetup>,
    screenshots: &mut Vec<RgbaImage>,
) -> Option<RegionCapture> {
    // Get the glfw instance. Keep looping until we get it because this is made during setup.
    let glfw = loop {
        let read_guard = GLFW_SETUP.read().unwrap();
        if let Some(glfw) = &*read_guard {
            let ptr = unsafe { &mut *glfw.data };
            drop(read_guard);
            break ptr;
        }
        drop(read_guard);
        std::thread::sleep(std::time::Duration::from_millis(1));
    };
    let bypass_box = SendSyncBypass::new(glfw);

    // Setup the region selector context.
    let mut ctx = match main_thread_sync(|| setup_region_selector(setup, screenshots, bypass_box)) {
        Some(ctx) => ctx,
        None => return None,
    };

    // Call the event loop handler in the main thread. Pull the result into the worker thread.
    let res = loop {
        // Run the event loop handler.
        let res = main_thread_sync(|| region_selector_event_loop_handler(&mut ctx));
        if let Some(res) = res {
            break res;
        }

        // Sleep for a couple of milliseconds so we don't hog the event loop.
        std::thread::sleep(std::time::Duration::from_millis(2));
    };

    // Clean up by making sure the context is dropped on the main thread.
    main_thread_drop(ctx);

    // If there is a result, reverse the image and return it.
    if let Some(mut res) = res {
        // Do this with the Rust image crate.
        image::imageops::flip_vertical_in_place(&mut res.image);

        // Return the result.
        Some(res)
    } else {
        None
    }
}
