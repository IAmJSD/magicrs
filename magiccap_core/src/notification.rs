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
    use cacao::foundation::{id, NSString, nil};
    use objc::{sel, msg_send, sel_impl, class};

    // Figure out the identifier.
    let identifier = match url {
        Some(url) => NSString::new(format!("url={}", url).as_str()),
        None => {
            match file_path {
                Some(file_path) => NSString::new(format!("fp={}", file_path).as_str()),
                None => NSString::new("none"),
            }
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
