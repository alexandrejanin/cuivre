use gl;
use image;
use maths::Vector2u;
use resources::Loadable;
use std::{cmp::Ordering, error, fmt, io};

/// ID of loaded OpenGL Texture
pub type TextureID = gl::types::GLuint;

/// Errors related to texture handling.
#[derive(Debug)]
pub enum TextureError {
    Io(io::Error),
    /// Error related to image handling.
    ImageError(image::ImageError),
    /// Tried creating texture from invalid data.
    /// Contains texture width, height, and data length.
    InvalidTextureData(u32, u32, u32, usize),
}

impl From<io::Error> for TextureError {
    fn from(error: io::Error) -> Self {
        TextureError::Io(error)
    }
}

impl error::Error for TextureError {}

impl fmt::Display for TextureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TextureError::Io(error) => write!(f, "{}", error),
            TextureError::ImageError(error) => write!(f, "{}", error),
            TextureError::InvalidTextureData(pixel_size, width, height, len) => write!(
                f,
                //TODO: nicer message
                "TextureError: {}x{}x{} != {}",
                pixel_size, width, height, len
            ),
        }
    }
}

/// Texture format.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TextureFormat {
    Rgb = gl::RGB as isize,
    Rgba = gl::RGBA as isize,
}

impl TextureFormat {
    /// Length of one pixel in a texture of this format,
    /// in bytes.
    pub fn pixel_length(self) -> u32 {
        match self {
            TextureFormat::Rgb => 3,
            TextureFormat::Rgba => 4,
        }
    }
}

/// Texture wrap mode.
///
/// Default: `Repeat`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum WrapMode {
    ClampToEdge = gl::CLAMP_TO_EDGE as isize,
    ClampToBorder = gl::CLAMP_TO_BORDER as isize,
    MirroredRepeat = gl::MIRRORED_REPEAT as isize,
    Repeat = gl::REPEAT as isize,
    MirrorClampToEdge = gl::MIRROR_CLAMP_TO_EDGE as isize,
}

/// Texture minification filtering mode.
///
/// Default: `NearestMipmapNearest`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MinFilterMode {
    Nearest = gl::NEAREST as isize,
    Linear = gl::LINEAR as isize,
    NearestMipmapNearest = gl::NEAREST_MIPMAP_NEAREST as isize,
    LinearMipmapNearest = gl::LINEAR_MIPMAP_NEAREST as isize,
    NearestMipmapLinear = gl::NEAREST_MIPMAP_LINEAR as isize,
    LinearMipmapLinear = gl::LINEAR_MIPMAP_LINEAR as isize,
}

/// Texture magnification filtering mode.
///
/// Default: `Nearest`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaxFilterMode {
    Nearest = gl::NEAREST as isize,
    Linear = gl::LINEAR as isize,
}

/// Options for texture display.
#[derive(Debug, Copy, Clone)]
pub struct TextureOptions {
    pub format: TextureFormat,
    pub h_wrap_mode: WrapMode,
    pub v_wrap_mode: WrapMode,
    pub min_filter_mode: MinFilterMode,
    pub max_filter_mode: MaxFilterMode,
}

impl Default for TextureOptions {
    fn default() -> Self {
        Self {
            format: TextureFormat::Rgba,
            h_wrap_mode: WrapMode::Repeat,
            v_wrap_mode: WrapMode::Repeat,
            min_filter_mode: MinFilterMode::NearestMipmapNearest,
            max_filter_mode: MaxFilterMode::Nearest,
        }
    }
}

/// Contains ID and metadata of a texture loaded in OpenGL.
///
/// This owns the texture, meaning the OpenGL texture is deleted when
/// `Texture` goes out of scope.
///
/// NOTE: Make sure to create a `GraphicsManager` before creating any
/// texture, otherwise OpenGL won't be loaded properly!
#[derive(Debug)]
pub struct Texture {
    id: TextureID,
    size: Vector2u,
    options: TextureOptions,
}

impl Loadable for Texture {
    type LoadOptions = TextureOptions;
    type LoadError = TextureError;

    fn load_from_bytes(data: &[u8], options: TextureOptions) -> Result<Self, TextureError> {
        //Load image from bytes
        let img = image::load_from_memory(data)
            .map_err(TextureError::ImageError)?
            .to_rgba();
        let (width, height) = img.dimensions();

        Self::from_bytes(img.as_ref(), options, width, height)
    }
}

impl PartialEq for Texture {
    fn eq(&self, other: &Texture) -> bool {
        self.id == other.id
    }
}

impl Eq for Texture {}

impl PartialOrd for Texture {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Sorting for drawcall batching.
impl Ord for Texture {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}

impl Texture {
    /// ID of the loaded texture in OpenGL.
    pub fn id(&self) -> TextureID {
        self.id
    }

    /// Size of the texture in pixels.
    pub fn size(&self) -> Vector2u {
        self.size
    }

    /// Width of the texture in pixels.
    pub fn width(&self) -> u32 {
        self.size.x
    }

    /// Height of the texture in pixels.
    pub fn height(&self) -> u32 {
        self.size.y
    }

    pub fn options(&self) -> &TextureOptions {
        &self.options
    }

    /// Create texture from raw pixel data.
    pub fn from_bytes(
        data: &[u8],
        options: TextureOptions,
        width: u32,
        height: u32,
    ) -> Result<Self, TextureError> {
        if data.len() != (options.format.pixel_length() * width * height) as usize {
            return Err(TextureError::InvalidTextureData(options.format.pixel_length(), width, height, data.len()));
        }

        //Allocate texture
        let mut id = 0;

        unsafe {
            //Create texture
            gl::GenTextures(1, &mut id);

            //Bind texture
            gl::BindTexture(gl::TEXTURE_2D, id);

            //Fill texture
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as gl::types::GLint,
                width as gl::types::GLint,
                height as gl::types::GLint,
                0,
                options.format as gl::types::GLenum,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const gl::types::GLvoid,
            );

            //Texture wrapping
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                options.h_wrap_mode as gl::types::GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                options.v_wrap_mode as gl::types::GLint,
            );

            //Texture filtering
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                options.min_filter_mode as gl::types::GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                options.max_filter_mode as gl::types::GLint,
            );

            //Generate mipmaps if min_filter_mode requires it
            match options.min_filter_mode {
                MinFilterMode::NearestMipmapNearest
                | MinFilterMode::LinearMipmapNearest
                | MinFilterMode::NearestMipmapLinear
                | MinFilterMode::LinearMipmapLinear => gl::GenerateMipmap(gl::TEXTURE_2D),
                MinFilterMode::Nearest | MinFilterMode::Linear => {}
            }

            //Unbind texture
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(Self {
            id,
            size: Vector2u::new(width, height),
            options,
        })
    }
}
