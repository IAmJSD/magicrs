struct Keyframe {
    data: Vec<u8>,
    usage_count: usize,
}

pub struct RGBACompressor {
    current_keyframe: *mut Keyframe,
    phantom: std::marker::PhantomData<()>,
}

pub struct RGBAFrame {
    keyframe_ptr: *mut Keyframe,
    patches: Vec<(usize, Vec<u8>)>,
    phantom: std::marker::PhantomData<()>,
}

impl RGBAFrame {
    pub fn decompress_into_buffer(&self, v: &mut Vec<u8>) {
        let keyframe = unsafe { &*self.keyframe_ptr };
        v.clear();
        v.extend_from_slice(&keyframe.data);
        for (offset, patch) in &self.patches {
            // Copy the patch into the buffer using memcpy.
            unsafe {
                libc::memcpy(
                    v.as_mut_ptr().add(*offset) as *mut libc::c_void,
                    patch.as_ptr() as *const libc::c_void,
                    patch.len(),
                )
            };
        }
    }
}

impl Drop for RGBAFrame {
    fn drop(&mut self) {
        unsafe {
            let keyframe = &mut *self.keyframe_ptr;
            keyframe.usage_count -= 1;
            if keyframe.usage_count == 0 {
                drop(Box::from_raw(self.keyframe_ptr));
            }
        }
    }
}

impl RGBACompressor {
    pub fn new() -> Self {
        Self {
            current_keyframe: std::ptr::null_mut(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn compress(&mut self, v: &mut Vec<u8>) -> RGBAFrame {
        if self.current_keyframe.is_null() {
            // We are the first write. Create a new keyframe.
            let keyframe = Box::new(Keyframe {
                data: v.clone(),
                usage_count: 1,
            });
            self.current_keyframe = Box::into_raw(keyframe);
            return RGBAFrame {
                keyframe_ptr: self.current_keyframe,
                patches: Vec::new(),
                phantom: std::marker::PhantomData,
            };
        }

        // Figure out the diff between the current keyframe and the new data.
        let keyframe = unsafe { &mut *self.current_keyframe };
        let mut diff = 0;
        let mut patches = Vec::new();
        let mut diff_start_index = -1;
        for (i, (a, b)) in keyframe.data.iter().zip(v.iter()).enumerate() {
            let diff_cont = a != b;
            if diff_start_index != -1 {
                if diff_cont {
                    // The difference is still continuing.
                    diff += 1;
                } else {
                    // The difference is done.
                    patches.push((
                        diff_start_index as usize,
                        v[diff_start_index as usize..i].to_vec(),
                    ));
                    diff_start_index = -1;
                }
            } else {
                if diff_cont {
                    // A new difference has started.
                    diff_start_index = i as i32;
                    diff += 1;
                }
            }
        }
        if diff_start_index != -1 {
            patches.push((
                diff_start_index as usize,
                v[diff_start_index as usize..].to_vec(),
            ));
        }

        // If the diff is more than 10% of the data, create a new keyframe.
        if diff as f32 / keyframe.data.len() as f32 > 0.1 {
            let keyframe = Box::new(Keyframe {
                data: v.clone(),
                usage_count: 1,
            });
            self.current_keyframe = Box::into_raw(keyframe);
            return RGBAFrame {
                keyframe_ptr: self.current_keyframe,
                patches: Vec::new(),
                phantom: std::marker::PhantomData,
            };
        }

        // Add a usage to the current keyframe and return the patches.
        keyframe.usage_count += 1;
        RGBAFrame {
            keyframe_ptr: self.current_keyframe,
            patches,
            phantom: std::marker::PhantomData,
        }
    }
}
