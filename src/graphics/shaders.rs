use super::ProgramID;
use cgmath::{Array, Matrix};
use gl;
use maths::{Matrix4f, Vector2f, Vector4f};
use resources::{self, ResourceLoader};
use std::{
    self,
    ffi::{self, CStr, CString},
    fmt::{self, Display, Formatter},
    path::Path,
};

///Errors related to shaders.
#[derive(Debug)]
pub enum ShaderError {
    NulError(ffi::NulError),
    ///An error related to resources handling.
    ResourceError(resources::ResourceError),
    ///OpenGL Shader could not compile. Contains OpenGL Error log.
    ShaderCompilationFailed(String),
    ///OpenGL Program could not link. Contains OpenGL Error log.
    ProgramLinkingFailed(String),
    ///Uniform was not found in the current program. Contains uniform name.
    InvalidUniform(String),
}

impl From<resources::ResourceError> for ShaderError {
    fn from(error: resources::ResourceError) -> Self {
        ShaderError::ResourceError(error)
    }
}

impl From<ffi::NulError> for ShaderError {
    fn from(error: ffi::NulError) -> Self {
        ShaderError::NulError(error)
    }
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Shader error: ")?;
        match self {
            ShaderError::NulError(error) => write!(f, "{}", error),
            ShaderError::ResourceError(error) => write!(f, "{}", error),
            ShaderError::ShaderCompilationFailed(message) => {
                write!(f, "Shader could not compile: {}", message)
            }
            ShaderError::ProgramLinkingFailed(message) => {
                write!(f, "Program could not link: {}", message)
            }
            ShaderError::InvalidUniform(uniform) => write!(f, "Invalid uniform: {}", uniform),
        }
    }
}

impl std::error::Error for ShaderError {
    fn cause(&self) -> Option<&std::error::Error> {
        match self {
            ShaderError::NulError(error) => Some(error),
            ShaderError::ResourceError(error) => Some(error),

            _ => None,
        }
    }
}

/// Represents an OpenGL shader program.
/// Required for drawing anything to the screen.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Program {
    id: ProgramID,
}

impl Program {
    ///Get the underlying program ID.
    pub fn id(self) -> ProgramID {
        self.id
    }

    ///Use this program for drawing.
    pub fn set_used(self) {
        unsafe {
            gl::UseProgram(self.id());
        }
    }

    ///Set a uniform mat4.
    pub fn set_mat4(self, name: &str, mat4: Matrix4f) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat4.as_ptr());
        }

        Ok(())
    }

    ///Set a uniform mat4 array.
    pub fn set_mat4_arr(self, name: &str, mat4s: &[Matrix4f]) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::UniformMatrix4fv(
                loc,
                mat4s.len() as gl::types::GLint,
                gl::FALSE,
                mat4s[0].as_ptr(),
            );
        }

        Ok(())
    }

    ///Set a uniform vec2.
    pub fn set_vec2(self, name: &str, vec2: Vector2f) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::Uniform2fv(loc, 1 as gl::types::GLint, vec2.as_ptr());
        }

        Ok(())
    }

    ///Set a uniform vec2 array.
    pub fn set_vec2_arr(self, name: &str, vec2s: &[Vector2f]) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::Uniform2fv(loc, vec2s.len() as gl::types::GLint, vec2s[0].as_ptr());
        }

        Ok(())
    }

    ///Set a uniform vec3.
    pub fn set_vec3(self, name: &str, vec3: Vector4f) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::Uniform3fv(loc, 1 as gl::types::GLint, vec3.as_ptr());
        }

        Ok(())
    }

    ///Set a uniform vec3 array.
    pub fn set_vec3_arr(self, name: &str, vec3s: &[Vector4f]) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::Uniform3fv(loc, vec3s.len() as gl::types::GLint, vec3s[0].as_ptr());
        }

        Ok(())
    }

    ///Set a uniform vec4.
    pub fn set_vec4(self, name: &str, vec4: Vector4f) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::Uniform4fv(loc, 1 as gl::types::GLint, vec4.as_ptr());
        }

        Ok(())
    }

    ///Set a uniform vec4 array.
    pub fn set_vec4_arr(self, name: &str, vec4s: &[Vector4f]) -> Result<(), ShaderError> {
        let loc = self.get_uniform_location(name)?;

        unsafe {
            gl::Uniform4fv(loc, vec4s.len() as gl::types::GLint, vec4s[0].as_ptr());
        }

        Ok(())
    }

    ///Returns uniform location in program from uniform name.
    fn get_uniform_location(self, name: &str) -> Result<gl::types::GLint, ShaderError> {
        let uniform_name = CString::new(name)?;

        let loc = unsafe { gl::GetUniformLocation(self.id, uniform_name.as_ptr()) };

        if loc < 0 {
            Err(ShaderError::InvalidUniform(name.into()))
        } else {
            Ok(loc)
        }
    }

    ///Create Program from vertex and fragment shader paths.
    pub fn load_shaders(
        resource_loader: &ResourceLoader,
        vertex_path: &Path,
        fragment_path: &Path,
    ) -> Result<Program, ShaderError> {
        Program::from_shaders(
            Shader::from_file(resource_loader, gl::VERTEX_SHADER, vertex_path)?,
            Shader::from_file(resource_loader, gl::FRAGMENT_SHADER, fragment_path)?,
        )
    }

    ///Create Program from Shaders. Deletes shaders afterwards.
    fn from_shaders(
        vertex_shader: Shader,
        fragment_shader: Shader,
    ) -> Result<Program, ShaderError> {
        let program_id = unsafe { gl::CreateProgram() };

        unsafe {
            gl::AttachShader(program_id, vertex_shader.id());
            gl::AttachShader(program_id, fragment_shader.id());
            gl::LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut error_length: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut error_length);
            }

            let error = empty_cstring(error_length as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    error_length,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(ShaderError::ProgramLinkingFailed(
                error.to_string_lossy().into_owned(),
            ));
        }

        unsafe {
            gl::DetachShader(program_id, vertex_shader.id());
            gl::DetachShader(program_id, fragment_shader.id());
            gl::DeleteShader(vertex_shader.id());
            gl::DeleteShader(fragment_shader.id());
        }

        Ok(Program { id: program_id })
    }
}

///Represents an OpenGL shader.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    ///Gets the Shader's OpenGL shader ID.
    fn id(self) -> gl::types::GLuint {
        self.id
    }

    ///Creates shader from source file.
    ///shader_type: usually gl::VERTEX_SHADER or gl::FRAGMENT_SHADER
    pub fn from_file(
        resource_loader: &ResourceLoader,
        shader_type: gl::types::GLuint,
        path: &Path,
    ) -> Result<Shader, ShaderError> {
        let text = CString::new(resource_loader.load_string(path)?)?;

        Shader::from_source(shader_type, &text)
    }

    ///Create a new shader from GLSL source (provided as a CString), returns Shader object or OpenGL error log.
    ///shader_type: usually gl::VERTEX_SHADER or gl::FRAGMENT_SHADER
    fn from_source(shader_type: gl::types::GLuint, source: &CStr) -> Result<Shader, ShaderError> {
        //Create shader and get ID
        let id = unsafe { gl::CreateShader(shader_type) };

        //Compile shader from source
        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        //Check shader compile status
        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        //Shader compiled successfully
        if success == 1 {
            return Ok(Shader { id });
        }

        //Compilation failed, get error message
        let mut error_length: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut error_length);
        }

        //Allocate CString for error log
        let error_log = empty_cstring(error_length as usize);

        //Fill error log
        unsafe {
            gl::GetShaderInfoLog(
                id,
                error_length,
                std::ptr::null_mut(),
                error_log.as_ptr() as *mut gl::types::GLchar,
            );
        }

        //Return error log
        Err(ShaderError::ShaderCompilationFailed(
            error_log.to_string_lossy().into_owned(),
        ))
    }
}

///Creates and returns a CString filled with 'length' spaces.
fn empty_cstring(length: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(length as usize + 1);
    buffer.extend([b' '].iter().cycle().take(length as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}
