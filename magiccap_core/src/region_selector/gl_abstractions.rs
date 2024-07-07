use std::ffi::c_uint;

// Defines a higher level API for managing textures.
pub struct GLTexture {
    pub texture: c_uint,
}

impl GLTexture {
    // Creates a new texture from a image::RgbaImage.
    pub fn from_rgba(image: &image::RgbaImage) -> GLTexture {
        // Generate the texture.
        let mut texture = 0;
        unsafe { gl::GenTextures(1, &mut texture) };

        // Bind the texture.
        unsafe { gl::BindTexture(gl::TEXTURE_2D, texture) };

        // Load the image.
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA as i32,
                image.width() as i32, image.height() as i32,
                0, gl::RGBA, gl::UNSIGNED_BYTE,
                image.as_ptr() as *const _
            );
        }

        // Unbind the texture.
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) };

        // Return the texture.
        GLTexture { texture }
    }

    // Creates a new texture from a raw pointer and type.
    pub fn from_raw(
        width: u32, height: u32, data: *const u8, data_type: i32,
    ) -> GLTexture {
        // Generate the texture.
        let mut texture = 0;
        unsafe { gl::GenTextures(1, &mut texture) };

        // Bind the texture.
        unsafe { gl::BindTexture(gl::TEXTURE_2D, texture) };

        // Load the image.
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, data_type,
                width as i32, height as i32,
                0, data_type as u32, gl::UNSIGNED_BYTE,
                data as *const _
            );
        }

        // Unbind the texture.
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) };

        // Return the texture.
        GLTexture { texture }
    }
}

// Ensures the texture is freed.
impl Drop for GLTexture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.texture) };
    }
}
