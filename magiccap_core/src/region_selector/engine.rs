use std::ffi::{c_int, CString};

#[repr(C)]
pub struct region_coordinate_t {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct region_result_t {
    pub coordinate: region_coordinate_t,
    pub w: u32,
    pub h: u32,
    pub rgba: *const u8,
    pub rgba_len: usize,
    pub display_index: c_int,
}

#[repr(C)]
pub struct screenshot_t {
    pub data: *const u8,
    pub w: usize,
    pub h: usize,
}

#[repr(C)]
pub struct gl_fragment_t {
    pub data: Option<CString>,
    pub name: Option<CString>,
    pub gl_object: u32,
}

extern "C" {
    pub fn region_selector_open(
        display_count: usize,
        coordinates: *const region_coordinate_t,
        screenshots: *const screenshot_t,
        fragments: *const gl_fragment_t,
        show_editors: bool,
    ) -> *const region_result_t;
}
