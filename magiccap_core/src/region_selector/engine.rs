use std::{collections::HashMap, thread};
use super::{
    event_loop_handler::region_selector_event_loop_handler,
    gl_abstractions::{GLShaderProgram, GLTexture},
    ui_renderer::region_selector_render_ui,
    RegionCapture,
};
use crate::mainthread::{main_thread_async, main_thread_sync};
use glfw::{Glfw, PWindow};
use include_dir::{include_dir, Dir};

// A container that holds data and the thread ID of the thread that created the container. Note that
// this container must be dropped in the same thread that created it, otherwise this will panic since
// the thread ID will be different.
pub struct ThreadBoundContainer<T> {
    data: T,
    thread_id: thread::ThreadId,
}

impl<T> ThreadBoundContainer<T> {
    // Creates a new container with the given data.
    fn new(data: T) -> ThreadBoundContainer<T> {
        ThreadBoundContainer {
            data,
            thread_id: thread::current().id(),
        }
    }

    // Gets a mutable reference to the data if the current thread is the same as the one that created the container.
    pub fn as_mut(&mut self) -> Result<&mut T, &'static str> {
        if self.thread_id == thread::current().id() {
            Ok(&mut self.data)
        } else {
            Err("Data accessed from wrong thread")
        }
    }
}

// This is safe to pass between threads because the data is not accessible from other threads.
unsafe impl<T> Send for ThreadBoundContainer<T> {}
unsafe impl<T> Sync for ThreadBoundContainer<T> {}

// Implements drop. Note that this will panic if the container is dropped in a different thread than the one that
// created it.
impl<T> Drop for ThreadBoundContainer<T> {
    fn drop(&mut self) {
        assert_eq!(self.thread_id, thread::current().id());
    }
}

// Defines the items required to setup the region selector.
pub struct RegionSelectorSetup {
    pub windows: Vec<xcap::Window>,
    pub monitors: Vec<xcap::Monitor>,
    pub images: Vec<image::RgbaImage>,
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
    pub gl_shaders: HashMap<String, GLShaderProgram>,
    pub gl_screenshots: Vec<GLTexture>,
}

// Include the directory with the shaders.
static SHADERS_FOLDER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/region_selector/fragments");

// Sets up the region selector.
fn setup_region_selector(setup: Box<RegionSelectorSetup>) -> Option<ThreadBoundContainer<RegionSelectorContext>> {
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

    // Compile the shaders.
    let mut gl_shaders = HashMap::new();
    for shader in SHADERS_FOLDER.files() {
        // This long line gets the shader name.
        let shader_name = shader.path().file_name().unwrap().
            to_str().unwrap().split(".").next().unwrap().to_string();

        // Read the shader.
        let shader = shader.contents_utf8().unwrap().to_string();

        // Compile the shader.
        let mut program = GLShaderProgram::new();
        program.compile_fragment_shader(shader);

        // Insert the shader into the hashmap.
        gl_shaders.insert(shader_name, program);
    }

    // Turn the images into textures.
    let gl_screenshots = setup.images.iter().map(|img| {
        GLTexture::from_rgba(&img)
    }).collect::<Vec<_>>();

    // Create the context.
    let mut context = RegionSelectorContext {
        setup,
        glfw,
        glfw_windows,
        glfw_events,
        gl_shaders,
        gl_screenshots,
    };

    // Render the UI.
    unsafe {
        region_selector_render_ui(
            &mut context, true, None
        )
    };

    // Return the context.
    Some(ThreadBoundContainer::new(context))
}

// Make sure a item gets dropped on the main thread.
fn main_thread_drop<T>(item: T)
where
    T: Send + 'static,
{
    main_thread_async(move || { drop(item) });
}

// Invokes the engine.
pub fn invoke(setup: Box<RegionSelectorSetup>) -> Option<RegionCapture> {
    // Setup the region selector context.
    let mut ctx = match main_thread_sync(|| setup_region_selector(setup)) {
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
