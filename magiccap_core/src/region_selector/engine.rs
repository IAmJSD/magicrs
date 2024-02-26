use std::thread;
use super::{
    event_loop_handler::region_selector_event_loop_handler,
    gl_abstractions::{GLShaderProgram, GLTexture},
    ui_renderer::region_selector_render_ui,
    RegionCapture,
};
use crate::mainthread::{main_thread_async, main_thread_sync};
use glfw::{Glfw, PWindow};
use image::RgbaImage;
use include_dir::{include_dir, Dir};

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
    pub fn as_mut(&mut self) -> &mut T { &mut self.data }
}
unsafe impl<T> Send for SendSyncBypass<T> {}
unsafe impl<T> Sync for SendSyncBypass<T> {}

// Defines the items required to setup the region selector.
pub struct RegionSelectorSetup {
    pub windows: Vec<xcap::Window>,
    pub monitors: Vec<xcap::Monitor>,
    pub show_editors: bool,
}

// Doesn't actually matter here.
unsafe impl Send for RegionSelectorSetup {}

// Defines the setup results.
pub struct RegionSelectorContext {
    pub setup: Box<RegionSelectorSetup>,
    pub glfw: Glfw,
    pub glfw_windows: Vec<PWindow>,
    pub glfw_events: Vec<glfw::GlfwReceiver<(f64, glfw::WindowEvent)>>,
    pub image_dimensions: Vec<(u32, u32)>,
    pub gl_screenshots: Vec<GLTexture>,
    pub gl_screenshots_darkened: Vec<GLTexture>,
}

// Include the directories with the shaders.
static FRAGMENT_SHADERS_FOLDER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/region_selector/fragments");
static VERTEX_SHADERS_FOLDER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/region_selector/vertexes");

// Sets up the region selector.
fn setup_region_selector(setup: Box<RegionSelectorSetup>, screenshots: &mut Vec<RgbaImage>) -> Option<SendSyncBypass<RegionSelectorContext>> {
    // Setup glfw.
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // Go through each monitor and create a window for it.
    let mut glfw_windows: Vec<PWindow> = Vec::with_capacity(setup.monitors.len());
    let mut glfw_events = Vec::with_capacity(setup.monitors.len());
    if !glfw.with_connected_monitors(|glfw, glfw_monitors| {
        for monitor in &setup.monitors {
            // Find the matching glfw monitor.
            let glfw_monitor = glfw_monitors.iter().
                find(|m| m.get_pos() == (monitor.x(), monitor.y())).unwrap();

            // Set the window hints to control the version of OpenGL.
            glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
            glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
            glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

            // Create the window.
            let (window, events) = match glfw.create_window(
                monitor.width(), monitor.height(), "Region Selector", glfw::WindowMode::FullScreen(&glfw_monitor),
            ) {
                Some((window, events)) => (window, events),
                None => {
                    for window in &mut glfw_windows {
                        window.set_should_close(true);
                    }
                    return false;
                },
            };

            // Push the window and events.
            glfw_windows.push(window);
            glfw_events.push(events);
        }

        // Return true since success.
        true
    }) { return None; }

    // If there is no windows, return None.
    if glfw_windows.is_empty() { return None; }

    // Load in the OpenGL functions.
    let first_window_ref = &mut glfw_windows[0];
    gl::load_with(|s| first_window_ref.get_proc_address(s) as *const _);

    // Compile the brightness shader.
    let brightness_frag = FRAGMENT_SHADERS_FOLDER.get_file("brightness.frag").unwrap().contents_utf8().unwrap().to_string();
    let brightness_vert = VERTEX_SHADERS_FOLDER.get_file("brightness.vert").unwrap().contents_utf8().unwrap().to_string();
    let mut gl_brightness_program = GLShaderProgram::new();
    gl_brightness_program.compile_fragment_shader(brightness_frag, "brightness.frag");
    gl_brightness_program.compile_vertex_shader(brightness_vert, "brightness.vert");
    gl_brightness_program.link();

    // Get the image dimensions.
    let image_dimensions = screenshots.iter().map(|img| {
        img.dimensions()
    }).collect::<Vec<_>>();

    // Turn the images into textures.
    let gl_screenshots = screenshots.iter().map(|img| {
        GLTexture::from_rgba(&img)
    }).collect::<Vec<_>>();

    // Turn the images into darkened textures by manipulating the underlying data.
    // This is quicker than compiling a shader on first load and since we are not
    // mutating it in OpenGL, it will be very fast to blit from the texture.
    let gl_screenshots_darkened = screenshots.iter_mut().map(
        |img| {
            // Darken the image.
            super::image_manipulation_simd::set_brightness_half_simd(img);

            // Create the texture.
            GLTexture::from_rgba(&img)
        }
    ).collect::<Vec<_>>();

    // Create the context.
    let mut context = RegionSelectorContext {
        setup,
        glfw,
        glfw_windows,
        glfw_events,
        image_dimensions,
        gl_screenshots,
        gl_screenshots_darkened,
    };

    // Render the UI.
    unsafe {
        region_selector_render_ui(
            &mut context, true, None
        )
    };

    // Return the context.
    Some(SendSyncBypass::new(context))
}

// Make sure a item gets dropped on the main thread.
fn main_thread_drop<T>(item: T)
where
    T: Send + 'static,
{
    main_thread_async(move || { drop(item) });
}

// Invokes the engine.
pub fn invoke(setup: Box<RegionSelectorSetup>, screenshots: &mut Vec<RgbaImage>) -> Option<RegionCapture> {
    // Setup the region selector context.
    let mut ctx = match main_thread_sync(|| setup_region_selector(
        setup, screenshots,
    )) {
        Some(ctx) => ctx,
        None => return None,
    };

    // Run the event loop.
    let res: Option<RegionCapture>;
    loop {
        // Call the event loop handler in the main thread. Pull the result into the worker thread.
        match main_thread_sync(|| region_selector_event_loop_handler(&mut ctx)) {
            Some(v) => {
                res = v;
                break;
            },
            None => {},
        };

        // Sleep for 1 second / 120fps.
        thread::sleep(std::time::Duration::from_millis(8));
    }

    // Clean up by making sure the context is dropped on the main thread.
    main_thread_drop(ctx);

    // Return the result.
    res
}
