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
    use crate::linux_shared::app;

    app().main_thread_writer.send(Box::new(handler));
}

// Handles the main thread synchronously on Linux.
#[cfg(target_os = "linux")]
pub fn main_thread_sync<F, T>(handler: F) -> T
where
    F: Send + FnOnce() -> T, T: Send
{
    use crate::linux_shared::app;
    use std::sync::mpsc::channel;

    // Box the handler into a raw pointer. This is to get around some Rust rules
    // that are not overly useful here.
    let handler_ptr = Box::into_raw(Box::new(handler)) as usize;

    // Defines the function handler.
    let (tx, rx) = channel();
    let sent_handler = Box::new(move || {
        let handler: Box<F> = unsafe { Box::from_raw(handler_ptr as *mut F) };
        let res = handler();
        tx.send(Box::into_raw(Box::new(res)) as usize).unwrap();
    });
    app().main_thread_writer.send(sent_handler);

    // Wait for the result and then return it.
    let res_ptr = rx.recv().unwrap();
    unsafe {
        let res = Box::from_raw(res_ptr as *mut T);
        *res
    }
}
