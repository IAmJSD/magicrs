use std::{ffi::{c_uint, CString}, ptr};

// Defines a higher level API for managing shader programs.
pub struct GLShaderProgram {
    pub program: c_uint,
}

impl GLShaderProgram {
    // Creates a new shader program.
    pub fn new() -> GLShaderProgram {
        GLShaderProgram {
            program: unsafe { gl::CreateProgram() },
        }
    }

    // Use the shader program.
    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.program) };
    }

    // Takes a fragment shader and compiles it.
    pub fn compile_fragment_shader(&mut self, source: String) {
        // Create the shader.
        let shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };

        // Compile the shader.
        let cstr = CString::new(source).unwrap();
        unsafe {
            gl::ShaderSource(
                shader, 1,
                &cstr.as_ptr(),
                ptr::null()
            );
            gl::CompileShader(shader);
        }
        drop(cstr);

        // Attach the shader to the program.
        unsafe { gl::AttachShader(self.program, shader) };

        // Delete the shader.
        unsafe { gl::DeleteShader(shader) };
    }
}

// Ensures the shader program is freed.
impl Drop for GLShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program) };
    }
}

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

        // Set the texture parameters.
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
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
