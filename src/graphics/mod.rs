use gl;
use maths::Vector2u;
use std::cmp::Ordering;

mod batches;
pub mod camera;
pub mod manager;
pub mod mesh;
pub mod shaders;
pub mod sprites;

///ID of loaded OpenGL Texture
pub type TextureID = gl::types::GLuint;

///ID of loaded OpenGL Program
pub type ProgramID = gl::types::GLuint;

///Represents a texture loaded in OpenGL.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    id: TextureID,
    size: Vector2u,
}

impl PartialOrd for Texture {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Texture {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Texture {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn size(&self) -> Vector2u {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.size.x
    }

    pub fn height(&self) -> u32 {
        self.size.y
    }
}
