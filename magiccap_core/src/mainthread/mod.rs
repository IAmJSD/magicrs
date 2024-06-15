// Handles the main thread asynchronously on macOS.
#[cfg(target_os = "macos")]
pub fn main_thread_async<F>(handler: F)
where
    F: FnOnce() + Send + 'static,
{
    use dispatch::Queue;

    Queue::main().exec_async(handler);
}

// Handles the main thread synchronously on macOS.
#[cfg(target_os = "macos")]
pub fn main_thread_sync<F, T>(handler: F) -> T
where
    F: Send + FnOnce() -> T, T: Send
{
    use dispatch::Queue;

    Queue::main().exec_sync(handler)
}

// Handles the main thread asynchronously on Linux.
#[cfg(target_os = "linux")]
pub fn main_thread_async<F>(handler: F)
where
    F: FnOnce() + Send + 'static,
{
    use glib::idle_add_once;

    idle_add_once(handler);
}

// Include the one usage channel module on Linux.
#[cfg(target_os = "linux")]
mod one_usage_channel;

// Handles the main thread synchronously on Linux.
#[cfg(target_os = "linux")]
pub fn main_thread_sync<F, T>(handler: F) -> T
where
    F: Send + FnOnce() -> T, T: Send
{
    use crate::mainthread::one_usage_channel::channel;

    // Box the handler into a raw pointer. This is to get around some Rust rules
    // that are not overly useful here.
    let handler_ptr = Box::into_raw(Box::new(handler)) as usize;

    // Defines the function handler.
    let (mut sender, reciever) = channel();
    main_thread_async(move || {
        let handler: Box<F> = unsafe { Box::from_raw(handler_ptr as *mut F) };
        let res = handler();
        sender.send(Box::into_raw(Box::new(res)) as usize);
    });

    // Wait for the result and then return it.
    let res_ptr = reciever.wait();
    unsafe {
        let res = Box::from_raw(res_ptr as *mut T);
        *res
    }
}
