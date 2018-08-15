use gl;
use maths::Vector2u;
use std::cmp::Ordering;

/// ID of loaded OpenGL Texture
pub type TextureID = gl::types::GLuint;

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
    NearestMipmapNearest = gl::NEAREST_MIPMAP_NEAREST as isize,
    LinearMipmapNearest = gl::LINEAR_MIPMAP_NEAREST as isize,
    NearestMipmapLinear = gl::NEAREST_MIPMAP_LINEAR as isize,
    LinearMipmapLinear = gl::LINEAR_MIPMAP_LINEAR as isize,
}

/// Texture magnification filtering mode.
///
/// Default: `NearestMipmapNearest`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaxFilterMode {
    Nearest = gl::NEAREST as isize,
    Linear = gl::LINEAR as isize,
}

/// Contains ID and metadata of a texture loaded in OpenGL.
///
/// This owns the texture, meaning the OpenGL texture is deleted when
/// `Texture` goes out of scope.
#[derive(Debug, PartialEq, Eq)]
pub struct Texture {
    id: TextureID,
    size: Vector2u,
    h_wrap_mode: WrapMode,
    v_wrap_mode: WrapMode,
    min_filter_mode: MinFilterMode,
    max_filter_mode: MaxFilterMode,
}

impl PartialOrd for Texture {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Sorting for drawcall batching.
impl Ord for Texture {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.id.cmp(&other.id) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            _ => {}
        }
        match self.h_wrap_mode.cmp(&other.h_wrap_mode) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            _ => {}
        }
        match self.v_wrap_mode.cmp(&other.v_wrap_mode) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            _ => {}
        }
        match self.min_filter_mode.cmp(&other.min_filter_mode) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            _ => {}
        }
        match self.max_filter_mode.cmp(&other.max_filter_mode) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            _ => {}
        }

        Ordering::Equal
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}

impl Texture {
    pub fn new(
        id: TextureID,
        size: Vector2u,
        h_wrap_mode: WrapMode,
        v_wrap_mode: WrapMode,
        min_filter_mode: MinFilterMode,
        max_filter_mode: MaxFilterMode,
    ) -> Self {
        Self {
            id,
            size,
            h_wrap_mode,
            v_wrap_mode,
            min_filter_mode,
            max_filter_mode,
        }
    }

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
}
