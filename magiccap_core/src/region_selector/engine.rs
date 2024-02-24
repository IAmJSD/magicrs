use std::{collections::HashMap, ffi::{c_uint, CString}, ptr, thread};
use super::{
    ui_renderer::region_selector_render_ui,
    event_loop_handler::region_selector_event_loop_handler,
    RegionCapture,
};
use xcap::Monitor;
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
    pub monitors: Vec<Monitor>,
    pub images: Vec<image::RgbaImage>,
    pub show_editors: bool,
}

// Doesn't actually matter here.
unsafe impl Send for RegionSelectorSetup {}

// Defines a high-level API for shader programs.
struct GLShaderProgram {
    pub program: c_uint,
}

impl GLShaderProgram {
    // Creates a new shader program.
    pub fn new() -> GLShaderProgram {
        GLShaderProgram {
            program: unsafe { gl::CreateProgram() },
        }
    }

    // Takes a fragment shader and compiles it.
    pub fn compile_fragment_shader(&mut self, source: String) {
        // Create the shader.
        let shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };

        // Compile the shader.
        let cstr = CString::new(source).unwrap();
        unsafe {
            gl::ShaderSource(
                shader, 1,
                &cstr.as_ptr(),
                ptr::null()
            );
            gl::CompileShader(shader);
        }
        drop(cstr);

        // Attach the shader to the program.
        unsafe { gl::AttachShader(self.program, shader) };

        // Delete the shader.
        unsafe { gl::DeleteShader(shader) };
    }
}

// Ensures the shader program is freed.
impl Drop for GLShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program) };
    }
}

// Defines the setup results.
pub struct RegionSelectorContext {
    pub setup: Box<RegionSelectorSetup>,
    pub glfw: Glfw,
    pub glfw_windows: Vec<PWindow>,
    pub glfw_events: Vec<glfw::GlfwReceiver<(f64, glfw::WindowEvent)>>,
    pub glfw_shaders: HashMap<String, GLShaderProgram>,
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
    let mut glfw_shaders = HashMap::new();
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
        glfw_shaders.insert(shader_name, program);
    }

    // Create the context.
    let mut context = RegionSelectorContext {
        setup,
        glfw,
        glfw_windows,
        glfw_events,
        glfw_shaders,
    };

    // Render the UI.
    region_selector_render_ui(&mut context, true);

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
