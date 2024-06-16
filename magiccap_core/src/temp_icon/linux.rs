use muda::MenuEvent;
use tray_icon::{TrayIcon, TrayIconBuilder, menu::{MenuItem, Menu}};
use crate::linux_shared::FakeSend;
use std::{sync::atomic::{AtomicBool, Ordering}, thread, io::Read};
use crate::temp_icon::shared::{STOP_ICON, COG_ICON};

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
    menu.append(&MenuItem::new("Stop Recording", true, None)).unwrap();
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
