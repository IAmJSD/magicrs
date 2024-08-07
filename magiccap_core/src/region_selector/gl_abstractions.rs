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
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.as_ptr() as *const _,
            );
        }

        // Unbind the texture.
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) };

        // Return the texture.
        GLTexture { texture }
    }

    // Creates a new texture from a image::RgbImage.
    pub fn from_rgb(image: &image::RgbImage) -> GLTexture {
        // Generate the texture.
        let mut texture = 0;
        unsafe { gl::GenTextures(1, &mut texture) };

        // Bind the texture.
        unsafe { gl::BindTexture(gl::TEXTURE_2D, texture) };

        // Load the image.
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                image.as_ptr() as *const _,
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
