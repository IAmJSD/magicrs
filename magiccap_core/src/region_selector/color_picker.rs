// Use GTK on Linux to pick a color.
#[cfg(target_os = "linux")]
pub fn open_color_picker(cb: impl FnOnce((u8, u8, u8)) + 'static) {
    use std::cell::RefCell;
    use std::rc::Rc;
    use gtk::prelude::*;
    use gtk::ColorChooserDialog;

    // Create the dialog.
    let dialog = ColorChooserDialog::new(Some("Pick a color"), None as Option<&gtk::Window>);

    // Make sure it is a modal dialog and always on top.
    dialog.set_modal(true);
    dialog.set_keep_above(true);

    // Make sure the dialog is destroyed when closed.
    let cb = Rc::new(RefCell::new(Some(cb)));
    let cb_clone = Rc::clone(&cb);
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Ok {
            let color = dialog.rgba();
            let result = (
                (color.red() * 255.0) as u8,
                (color.green() * 255.0) as u8,
                (color.blue() * 255.0) as u8,
            );
            let cb = cb_clone.borrow_mut().take().unwrap();
            cb(result);
        }
        dialog.close();
    });

    // Make sure it is always focused.
    dialog.connect_show(|dialog| {
        dialog.grab_focus();
    });

    // Show the dialog.
    dialog.show_all();
}

// Stub for now.
#[cfg(target_os = "windows")]
pub fn open_color_picker(cb: impl FnOnce((u8, u8, u8)) + 'static) {}
