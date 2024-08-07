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
    F: Send + FnOnce() -> T,
    T: Send,
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

// Create the main event loop on Windows.
#[cfg(target_os = "windows")]
pub fn main_event_loop() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageA, GetMessageA, TranslateMessage, MSG, WM_USER,
    };

    loop {
        let mut msg = MSG::default();
        let msg_mut_ptr = &mut msg as *mut MSG;
        let got_message = unsafe { GetMessageA(msg_mut_ptr, None, 0, 0) };
        if !got_message.as_bool() {
            return;
        }
        if msg.message == WM_USER {
            // Get the memory address of the function.
            let mem_address = msg.wParam.0;
            let (handler, addr) =
                *unsafe { Box::from_raw(mem_address as *mut (extern "C" fn(usize), usize)) };
            handler(addr);
        } else {
            // Let Windows handle this.
            let msg_ptr = &msg as *const MSG;
            unsafe {
                let _ = TranslateMessage(msg_ptr);
                DispatchMessageA(msg_ptr);
            }
        }
    }
}

// Handles a main thread push on Windows.
#[cfg(target_os = "windows")]
pub fn main_thread_async<F>(handler: F)
where
    F: FnOnce() + Send + 'static,
{
    use crate::windows_shared::app;
    use windows::Win32::{
        Foundation::WPARAM,
        UI::WindowsAndMessaging::{PostThreadMessageA, WM_USER},
    };

    extern "C" fn caller<F>(func: Box<F>)
    where
        F: FnOnce() + Send + 'static,
    {
        (*func)();
    }
    let func: extern "C" fn(Box<F>) = caller::<F>;
    let mem_addr = Box::into_raw(Box::new((func, Box::into_raw(Box::new(handler)))));
    unsafe {
        PostThreadMessageA(
            app().main_thread_id,
            WM_USER,
            WPARAM(mem_addr as usize),
            None,
        )
        .unwrap();
    }
}

// Handles the main thread synchronously on platforms that do not natively handle it.
#[cfg(not(target_os = "macos"))]
pub fn main_thread_sync<F, T>(handler: F) -> T
where
    F: Send + FnOnce() -> T,
    T: Send,
{
    use std::sync::mpsc::sync_channel;

    // Box the handler into a raw pointer. This is to get around some Rust rules
    // that are not overly useful here.
    let handler_ptr = Box::into_raw(Box::new(handler)) as usize;

    // Defines the function handler.
    let (sender, reciever) = sync_channel(1);
    main_thread_async(move || {
        let handler: Box<F> = unsafe { Box::from_raw(handler_ptr as *mut F) };
        let res = handler();
        sender.send(Box::into_raw(Box::new(res)) as usize).unwrap();
    });

    // Wait for the result and then return it.
    let res_ptr = reciever.recv().unwrap();
    unsafe {
        let res = Box::from_raw(res_ptr as *mut T);
        *res
    }
}
