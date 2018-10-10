use assets::Asset;
use failure::Error;
use gl;
use graphics::textures::{
    MaxFilterMode, MinFilterMode, Texture, TextureFormat, TextureOptions, WrapMode,
};
use maths::Vector4f;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rusttype::{
    self,
    gpu_cache::{Cache, CacheBuilder},
    Point, PositionedGlyph, Scale,
};
use std::{os::raw::c_void, sync::Arc};
use unicode_normalization::UnicodeNormalization;

const CACHE_WIDTH: u32 = 1024;
const CACHE_HEIGHT: u32 = 1024;

/// Settings for drawing text.
#[derive(Debug, Copy, Clone)]
pub struct TextSettings {
    pub scale: f32,
    pub color: (u8, u8, u8),
    pub line_width: u32,
}

/// Represents a rendered character's position.
#[derive(Debug, Copy, Clone)]
pub struct CharacterPosition {
    pub texture_position: Vector4f,
    pub world_position: Vector4f,
}

pub struct Font<'a> {
    font: rusttype::Font<'a>,
    cache: Cache<'a>,
    texture: Texture,
}

impl<'a> Asset<()> for Font<'a> {
    fn load_from_bytes(data: &[u8], options: ()) -> Result<Self, Error> {
        //Cache texture settings
        let options = TextureOptions {
            format: TextureFormat::Rgba,
            h_wrap_mode: WrapMode::Repeat,
            v_wrap_mode: WrapMode::Repeat,
            min_filter_mode: MinFilterMode::Linear,
            max_filter_mode: MaxFilterMode::Linear,
        };

        let texture = Texture::from_bytes(
            &[0xFF; (4 * CACHE_WIDTH * CACHE_HEIGHT) as usize],
            options,
            CACHE_WIDTH,
            CACHE_HEIGHT,
        )?;

        let arc = Arc::from(data);
        let cache = CacheBuilder {
            width: CACHE_WIDTH,
            height: CACHE_HEIGHT,
            scale_tolerance: 0.1,
            position_tolerance: 0.1,
            pad_glyphs: true,
        }
        .build();

        Ok(Self {
            font: rusttype::Font::from_bytes(arc)?,
            cache,
            texture,
        })
    }
}

impl<'a> Font<'a> {
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Returns a sequence of the position of each character in the string on the cache.
    pub fn get_glyphs(
        &mut self,
        text: &str,
        settings: TextSettings,
    ) -> Result<Vec<CharacterPosition>, Error> {
        let glyphs = Self::layout_paragraph(
            text,
            &self.font,
            Scale::uniform(settings.scale),
            settings.line_width,
        );

        let texture = &self.texture;

        let cache = &mut self.cache;

        for glyph in &glyphs {
            cache.queue_glyph(0, glyph.clone());
        }

        cache.cache_queued(|rect, data| unsafe {
            let mut rgb_data: Vec<u8> =
                Vec::with_capacity((4 * rect.width() * rect.height()) as usize);

            for alpha in data {
                rgb_data.push(settings.color.0);
                rgb_data.push(settings.color.1);
                rgb_data.push(settings.color.2);
                rgb_data.push(*alpha);
            }

            gl::TextureSubImage2D(
                texture.id(),
                0,
                rect.min.x as gl::types::GLint,
                rect.min.y as gl::types::GLint,
                rect.width() as gl::types::GLint,
                rect.height() as gl::types::GLint,
                texture.options().format as gl::types::GLenum,
                gl::UNSIGNED_BYTE,
                rgb_data.as_ptr() as *const c_void,
            );
        })?;

        //Get texture coordinates as Vector4f for each character
        let vec = glyphs
            .par_iter()
            .filter_map(
                |glyph| match cache.rect_for(0, glyph).expect("Could not read cache.") {
                    Some((tex_pos, world_pos)) => Some(CharacterPosition {
                        texture_position: Vector4f::new(
                            tex_pos.min.x,
                            tex_pos.min.y,
                            tex_pos.width(),
                            tex_pos.height(),
                        ),
                        world_position: Vector4f::new(
                            world_pos.min.x as f32 + (world_pos.width() as f32) / 2.0,
                            -(world_pos.min.y as f32 + (world_pos.height() as f32) / 2.0),
                            world_pos.width() as f32,
                            world_pos.height() as f32,
                        ) / 100.0,
                    }),
                    None => None,
                },
            )
            .collect();

        Ok(vec)
    }

    fn layout_paragraph<'b>(
        text: &str,
        font: &rusttype::Font<'b>,
        scale: Scale,
        width: u32,
    ) -> Vec<PositionedGlyph<'b>> {
        let mut result = Vec::new();

        let v_metrics = font.v_metrics(scale);

        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = Point {
            x: 0.0,
            y: v_metrics.ascent,
        };
        let mut last_glyph_id = None;

        for c in text.nfc() {
            //Special behavior for some characters
            if c.is_control() {
                if let '\n' = c {
                    caret.x = 0.0;
                    caret.y += advance_height;
                }
                continue;
            }

            let base_glyph = font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, base_glyph.id());
            }

            last_glyph_id = Some(base_glyph.id());
            let mut glyph = base_glyph.scaled(scale).positioned(caret);

            //Wrap lines
            if let Some(bb) = glyph.pixel_bounding_box() {
                if bb.max.x > width as i32 {
                    caret.x = 0.0;
                    caret.y += advance_height;

                    glyph = glyph.into_unpositioned().positioned(caret);
                    last_glyph_id = None;
                }
            }

            //Advance caret and push glyph
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            result.push(glyph);
        }
        result
    }
}
