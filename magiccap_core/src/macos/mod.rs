use std::os::raw::{c_char, c_int, c_void};

extern "C" {
    pub fn open_file_dialog(folder: bool) -> usize;
    pub fn copy_file_to_clipboard(
        file_path: *const c_char, filename: *const c_char,
        data: *const u8, data_len: usize,
    );
    pub fn send_ok_dialog(message: *const c_char);
    pub fn hook_notif_center();
    pub fn transform_process_type(show: bool);
    pub fn create_tray(
        uploader_items: *const UploaderItem, uploader_items_len: usize,
        capture_types: *const CaptureType, capture_types_len: usize,
        on_click: unsafe extern fn(
            name_ptr: *const u8, name_len: usize, path_ptr: *const u8, path_len: usize,
        ),
        on_quit: extern fn(),
        on_capture_type_clicked: extern fn(type_: c_int),
        on_config: extern fn(),
    ) -> usize;
}

#[repr(C)]
pub struct UploaderItem {
    pub name: *const c_char,
    pub id: *const c_char,
    pub default_uploader: bool,
}

#[repr(C)]
pub struct CaptureType {
    pub name: *const c_char,
    pub type_: c_int,
}
