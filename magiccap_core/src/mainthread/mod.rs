// Handles the main thread asynchronously on macOS.
#[cfg(target_os = "macos")]
pub fn main_thread_async<F>(handler: F)
where
    F: FnOnce() + Send + 'static,
{
    use dispatch::Queue;

    Queue::main().exec_async(handler);
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
