use super::Texture;
use maths::{Vector2f, Vector2i, Vector2u, Vector4f};

///Represents an OpenGL texture sliced into sprites.
#[derive(Debug)]
pub struct SpriteSheet<'t> {
    texture: &'t Texture,
    sprite_size: Vector2u,
    gl_size: Vector2f,
}

impl<'t> SpriteSheet<'t> {
    ///Create a new sprite sheet from a mesh (quad), texture and sprite size (in pixels)
    pub fn new(texture: &'t Texture, sprite_size: Vector2u) -> SpriteSheet<'t> {
        SpriteSheet {
            texture,
            sprite_size,
            gl_size: Vector2f::new(
                sprite_size.x as f32 / texture.width() as f32,
                sprite_size.y as f32 / texture.height() as f32,
            ),
        }
    }

    pub fn sprite(&self, x: i32, y: i32) -> Sprite {
        Sprite {
            sheet: self,
            position: Vector2i::new(x, y),
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
pub struct Sprite<'s> {
    sheet: &'s SpriteSheet<'s>,
    pub position: Vector2i,
}

impl<'s> Sprite<'s> {
    pub fn texture(&self) -> &'s Texture {
        self.sheet.texture
    }

    pub fn gl_position(&self) -> Vector4f {
        self.sheet.gl_position(self.position)
    }
}
