use crate::linux_shared::FakeSend;
use crate::temp_icon::shared::{COG_ICON, STOP_ICON};
use muda::MenuEvent;
use std::{
    io::{BufRead, Read},
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
};
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIcon, TrayIconBuilder,
};

// Defines the slot for the temporary icon demon sender.
static mut TRAY_SENDER: Option<&'static mut FakeSend<TrayIcon>> = None;

// Defines the recording information.
static mut RECORDING: AtomicBool = AtomicBool::new(true);

// Handle doing the tray updates.
fn tray_updates() {
    // Get the tray sender.
    let sent = unsafe { TRAY_SENDER.as_mut().unwrap() };

    // Update the tooltip.
    sent.value
        .set_tooltip(Some("MagicCap Processing Recording"))
        .unwrap();

    // Update the icon.
    sent.value.set_icon(Some(COG_ICON.clone())).unwrap();

    // Log that we are in the processing stage.
    println!("MAGICCAP_PROCESSING");
}

// Defines the function to stop recording on Linux.
fn stop_recording() {
    // Get the current state.
    let state = unsafe { RECORDING.load(Ordering::Relaxed) };

    // If we are recording, stop.
    if state {
        // Set the recording state to false.
        unsafe { RECORDING.store(false, Ordering::Relaxed) };

        // Get a spot on the main thread.
        glib::idle_add_local_once(tray_updates);
    }
}

// On Linux, define a temporary icon demon that will be used to display the icon in the tray.
// This is because of how GTK works.
pub fn icond() {
    // Defines the tray.
    let menu = Box::new(Menu::new());
    menu.append(&MenuItem::new("Stop Recording", true, None))
        .unwrap();
    let tray = TrayIconBuilder::new()
        .with_tooltip("MagicCap Recording")
        .with_icon(STOP_ICON.clone())
        .with_menu(menu)
        .build()
        .unwrap();

    // We leak the box in a fake sender so that it is 'static since this process is short lived anyway.
    unsafe {
        TRAY_SENDER = Some(Box::leak(Box::new(FakeSend { value: tray })));
    }

    // Defines a thread to handle stdio.
    thread::spawn(move || {
        loop {
            // Read a byte from stdin.
            let mut buffer = [0; 1];
            let _ = std::io::stdin().read(&mut buffer);

            // If it is 's', toggle the recording state.
            if buffer[0] == b's' {
                stop_recording();
            }
        }
    });

    // Handle the menu event loop.
    MenuEvent::set_event_handler(Some(move |_| {
        // This will be a click event. Stop recording now.
        stop_recording();
    }));

    // Call gtk::main.
    gtk::main();
}

// Defines the icon handler. When the icon is dropped, it will be removed from the tray.
pub struct IconHandler {
    process: Mutex<process::Child>,
}

// Build a child process of ourselves but with the MAGICCAP_INTERNAL_TEMP_ICON env var set.
fn create_icond(cb: Box<dyn FnOnce() + Send>) -> process::Child {
    // Get the current executable path.
    let path = std::env::current_exe().unwrap();

    // Create the process.
    let mut proc = process::Command::new(path)
        .env("MAGICCAP_INTERNAL_TEMP_ICON", "1")
        .stdout(process::Stdio::piped())
        .stdin(process::Stdio::piped())
        .spawn()
        .unwrap();

    // Spawn a thread to handle stdout.
    let mut stdout = proc.stdout.take().unwrap();
    thread::spawn(move || {
        loop {
            // Read to a new line and store it in a buffer.
            let mut buffer = String::new();
            let buffer = match std::io::BufReader::new(&mut stdout).read_line(&mut buffer) {
                Ok(_) => buffer,
                Err(_) => return,
            };

            // If the buffer contains "MAGICCAP_PROCESSING", call the callback and return.
            if buffer.contains("MAGICCAP_PROCESSING") {
                cb();
                return;
            }
        }
    });

    // Return the process.
    proc
}

// Defines the icon handler implementation.
impl IconHandler {
    // Create a new version of the icon handler. Note that the stop callback
    // will NOT be ran in the main thread in all cases.
    pub fn new(stop_callback: Box<dyn FnOnce() + Send>) -> Self {
        Self {
            process: Mutex::new(create_icond(stop_callback)),
        }
    }

    // Remove the icon from the tray.
    pub fn remove(&mut self) {
        let mut proc = self.process.lock().unwrap();
        let _ = proc.kill();
    }
}
