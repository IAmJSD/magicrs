use std::{ffi::{c_uint, CString}, ptr};

// Defines a higher level API for managing shader programs.
pub struct GLShaderProgram {
    pub program: c_uint,
}

// Ensures the shader program successfully compiles.
fn panic_if_shader_comp_fail(shader: u32, filename: &str) {
    // Get the log length.
    let mut len = 0;
    unsafe { gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len) };

    // Return if the shader compiled.
    if len == 0 { return }

    // Get the log.
    let mut log = vec![0; len as usize];
    unsafe {
        gl::GetShaderInfoLog(
            shader, len, ptr::null_mut(),
            log.as_mut_ptr() as *mut _
        );
    }

    // Panic the log.
    panic!("Error: Shader '{}' compilation failed: {}", filename, String::from_utf8_lossy(&log));
}

impl GLShaderProgram {
    // Creates a new shader program.
    pub fn new() -> GLShaderProgram {
        GLShaderProgram {
            program: unsafe { gl::CreateProgram() },
        }
    }

    // Takes a fragment shader and compiles it.
    pub fn compile_fragment_shader(&mut self, source: String, filename: &str) {
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

        // Ensure the shader compiled.
        panic_if_shader_comp_fail(shader, filename);

        // Attach the shader to the program.
        unsafe { gl::AttachShader(self.program, shader) };

        // Delete the shader.
        unsafe { gl::DeleteShader(shader) };
    }

    // Takes a vertex shader and compiles it.
    pub fn compile_vertex_shader(&mut self, source: String, filename: &str) {
        // Create the shader.
        let shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };

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

        // Ensure the shader compiled.
        panic_if_shader_comp_fail(shader, filename);

        // Attach the shader to the program.
        unsafe { gl::AttachShader(self.program, shader) };

        // Delete the shader.
        unsafe { gl::DeleteShader(shader) };
    }

    // Links the shader program.
    pub fn link(&mut self) {
        unsafe { gl::LinkProgram(self.program) };
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

// Copies the image data from one texture to another.
pub fn copy_texture(texture: &GLTexture, width: u32, height: u32, x: i32, y: i32) -> GLTexture {
    // Generate the texture.
    let mut new_texture = 0;
    unsafe { gl::GenTextures(1, &mut new_texture) };

    // Bind the texture.
    unsafe { gl::BindTexture(gl::TEXTURE_2D, new_texture) };

    // Check if glCopyImageSubData is supported (macOS is stuck at 4.1 :/).
    let supported = unsafe {
        let mut major = 0;
        let mut minor = 0;
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        major > 4 || (major == 4 && minor >= 3)
    };

    if supported {
        // Copy the texture.
        unsafe {
            gl::CopyImageSubData(
                texture.texture, gl::TEXTURE_2D, 0, x, y, 0,
                new_texture, gl::TEXTURE_2D, 0, 0, 0, 0,
                width as i32, height as i32, 1
            );
        }
    } else {
        // Get the texture data.
        let mut data = vec![0; (width * height * 4) as usize];
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture.texture);
            gl::GetTexImage(
                gl::TEXTURE_2D, 0, gl::RGBA, gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _
            );
        }

        // Set the texture parameters.
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA as i32,
                width as i32, height as i32,
                0, gl::RGBA, gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
    }

    // Set the texture parameters.
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    // Unbind the texture.
    unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) };

    // Return the texture.
    GLTexture { texture: new_texture }
}
