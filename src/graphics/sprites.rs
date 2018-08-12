use super::Texture;
use maths::{Vector2f, Vector2i, Vector2u, Vector4f};

///Represents an OpenGL texture sliced into sprites.
#[derive(Copy, Clone, Debug)]
pub struct SpriteSheet {
    texture: Texture,
    sprite_size: Vector2u,
    gl_size: Vector2f,
}

impl SpriteSheet {
    ///Create a new sprite sheet from a mesh (quad), texture and sprite size (in pixels)
    pub fn new(texture: Texture, sprite_size: Vector2u) -> SpriteSheet {
        SpriteSheet {
            texture,
            sprite_size,
            gl_size: Vector2f::new(
                sprite_size.x as f32 / texture.width() as f32,
                sprite_size.y as f32 / texture.height() as f32,
            ),
        }
    }

    pub fn sprite_size(&self) -> Vector2u {
        self.sprite_size
    }

    pub fn sprite_width(&self) -> u32 {
        self.sprite_size.x
    }

    pub fn sprite_height(&self) -> u32 {
        self.sprite_size.y
    }

    pub fn gl_position(&self, position: Vector2i) -> Vector4f {
        Vector4f::new(
            (self.sprite_width() as i32 * position.x) as f32 / self.texture.width() as f32,
            (self.sprite_height() as i32 * position.y) as f32 / self.texture.height() as f32,
            self.gl_size.x,
            self.gl_size.y,
        )
    }
}

///Represents part of a sprite sheet drawn on a quad.
#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    sheet: SpriteSheet,
    pub position: Vector2i,
}

impl Sprite {
    ///Create a new sprite from a sprite sheet and a position
    pub fn new(sheet: SpriteSheet, position: Vector2i) -> Sprite {
        Sprite { sheet, position }
    }

    pub fn texture(&self) -> Texture {
        self.sheet.texture
    }

    pub fn gl_position(&self) -> Vector4f {
        self.sheet.gl_position(self.position)
    }
}
