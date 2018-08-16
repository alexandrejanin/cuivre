use super::Texture;
use maths::{Vector2f, Vector4f};

/// Represents a texture sliced into rectangular sprites.
///
/// This consumes the `Texture`.
#[derive(Debug)]
pub struct SpriteSheet {
    texture: Texture,
    sprite_width: u32,
    sprite_height: u32,
    gl_size: Vector2f,
}

impl SpriteSheet {
    /// Creates a new sprite sheet from a texture and sprite size (in pixels).
    pub fn new(texture: Texture, sprite_width: u32, sprite_height: u32) -> SpriteSheet {
        SpriteSheet {
            sprite_width,
            sprite_height,
            gl_size: Vector2f::new(
                sprite_width as f32 / texture.width() as f32,
                sprite_height as f32 / texture.height() as f32,
            ),
            texture,
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Retrieves the sprite at selected position on the grid.
    pub fn sprite(&self, x: i32, y: i32) -> Sprite {
        Sprite { sheet: self, x, y }
    }

    pub fn sprite_width(&self) -> u32 {
        self.sprite_width
    }

    pub fn sprite_height(&self) -> u32 {
        self.sprite_height
    }

    pub fn gl_position(&self, x: i32, y: i32) -> Vector4f {
        Vector4f::new(
            (self.sprite_width() as i32 * x) as f32 / self.texture.width() as f32,
            (self.sprite_height() as i32 * y) as f32 / self.texture.height() as f32,
            self.gl_size.x,
            self.gl_size.y,
        )
    }
}

/// Represents one tile of a sprite sheet.
#[derive(Debug)]
pub struct Sprite<'s> {
    sheet: &'s SpriteSheet,
    pub x: i32,
    pub y: i32,
}

impl<'s> Sprite<'s> {
    /// Texture used by this sprite.
    pub fn texture(&self) -> &'s Texture {
        &self.sheet.texture
    }

    /// Position of the sprite on the texture
    /// as OpenGL coordinates.
    pub fn gl_position(&self) -> Vector4f {
        self.sheet.gl_position(self.x, self.y)
    }
}
