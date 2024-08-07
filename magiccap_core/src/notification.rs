// Send a dialog message to the user.
#[cfg(target_os = "macos")]
pub fn send_dialog_message(message: &str) {
    use crate::macos::send_ok_dialog;
    use std::ffi::CString;

    let c_message = CString::new(message).unwrap();
    unsafe { send_ok_dialog(c_message.as_ptr()) };
    drop(c_message);
}

// Send a notification to the user.
#[cfg(target_os = "macos")]
pub fn send_notification(message: &str, url: Option<&str>, file_path: Option<&str>) {
    use cacao::foundation::{id, nil, NSString};
    use objc::{class, msg_send, sel, sel_impl};

    // Figure out the identifier.
    let identifier = match url {
        Some(url) => NSString::new(format!("url={}", url).as_str()),
        None => match file_path {
            Some(file_path) => NSString::new(format!("fp={}", file_path).as_str()),
            None => NSString::new("none"),
        },
    };

    unsafe {
        // Create the notification.
        let content: id = msg_send![class!(UNMutableNotificationContent), new];
        let _: id = msg_send![content, setTitle: NSString::new("MagicCap")];
        let _: id = msg_send![content, setBody: NSString::new(message)];
        let sound: id = msg_send![class!(UNNotificationSound), defaultSound];
        let _: id = msg_send![content, setSound: sound];

        // Send the notification.
        let _: id = msg_send![
            class!(UNNotificationRequest), requestWithIdentifier: identifier
            content: content trigger: nil
        ];
    };
}

// Send a dialog message to the user.
#[cfg(not(target_os = "macos"))]
pub fn send_dialog_message(message: &str) {
    use native_dialog::{MessageDialog, MessageType};

    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("MagicCap")
        .set_text(message)
        .show_alert()
        .unwrap();
}

// Send a notification to the user.
#[cfg(not(target_os = "macos"))]
pub fn send_notification(message: &str, url: Option<&str>, file_path: Option<&str>) {
    use notify_rust::Notification;

    let mut notif = Notification::new();
    notif.summary(message);

    if let Some(_) = url {
        notif.action("open_url", "Open URL");
    }

    if let Some(_) = file_path {
        notif.action("open_fp", "Open File");
    }

    #[allow(unused_variables)]
    let res = notif.show().unwrap();
    #[cfg(not(target_os = "windows"))]
    res.wait_for_action(|e| {
        match e {
            "open_url" => open::that(url.unwrap()).unwrap(),
            "open_fp" => open::that(file_path.unwrap()).unwrap(),
            _ => {}
        };
    })
}
