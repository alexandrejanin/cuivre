use gl;
use graphics::textures::{
    MaxFilterMode, MinFilterMode, Texture, TextureError, TextureFormat, TextureOptions, WrapMode,
};
use maths::{Vector2f, Vector4f};
use resources::Loadable;
use rusttype::{
    self,
    gpu_cache::{Cache, CacheBuilder, CacheReadErr, CacheWriteErr},
    point, PositionedGlyph, Scale,
};
use std::{error, fmt, io, os::raw::c_void, sync::Arc};

const CACHE_WIDTH: u32 = 256;
const CACHE_HEIGHT: u32 = 256;

#[derive(Debug)]
pub enum FontError {
    CacheRead(CacheReadErr),
    CacheWrite(CacheWriteErr),
    Io(io::Error),
    Rusttype(rusttype::Error),
    Texture(TextureError),
}

impl From<io::Error> for FontError {
    fn from(error: io::Error) -> Self {
        FontError::Io(error)
    }
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FontError::CacheRead(error) => write!(f, "{}", error),
            FontError::CacheWrite(error) => write!(f, "{}", error),
            FontError::Io(error) => write!(f, "{}", error),
            FontError::Rusttype(error) => write!(f, "{}", error),
            FontError::Texture(error) => write!(f, "{}", error),
        }
    }
}

impl error::Error for FontError {}


pub struct CharacterPosition {
    pub texture_position: Vector4f,
    pub world_position: Vector2f,
}


pub struct Font<'a> {
    font: rusttype::Font<'a>,
    cache: Cache<'a>,
    texture: Texture,
}

impl<'a> Loadable for Font<'a> {
    type LoadOptions = ();
    type LoadError = FontError;

    fn load_from_bytes(data: &[u8], _options: ()) -> Result<Self, FontError> {
        Self::from_bytes(data)
    }
}

impl<'a> Font<'a> {
    fn from_bytes(data: &[u8]) -> Result<Self, FontError> {
        //Cache texture settings
        let options = TextureOptions {
            format: TextureFormat::Rgba,
            h_wrap_mode: WrapMode::Repeat,
            v_wrap_mode: WrapMode::Repeat,
            min_filter_mode: MinFilterMode::Nearest,
            max_filter_mode: MaxFilterMode::Nearest,
        };

        let texture = Texture::from_bytes(
            &[0xFF; (4 * CACHE_WIDTH * CACHE_HEIGHT) as usize],
            options,
            CACHE_WIDTH,
            CACHE_HEIGHT,
        ).map_err(FontError::Texture)?;

        let arc = Arc::from(data);
        let cache = CacheBuilder {
            width: CACHE_WIDTH,
            height: CACHE_HEIGHT,
            scale_tolerance: 0.1,
            position_tolerance: 0.1,
            pad_glyphs: true,
        }.build();

        Ok(Self {
            font: rusttype::Font::from_bytes(arc).map_err(FontError::Rusttype)?,
            cache,
            texture,
        })
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Returns a sequence of the position of each character in the string on the cache.
    ///
    /// Can fail if the byte data for this font is invalid.
    pub fn get_glyphs(&mut self, text: &str, scale: Vector2f, line_width: u32, (r, g, b): (u8, u8, u8)) -> Result<Vec<CharacterPosition>, FontError> {
        let glyphs = self.layout_paragraph(text, Scale { x: scale.x, y: scale.y }, line_width);

        let texture = &self.texture;

        let cache = &mut self.cache;

        for glyph in &glyphs {
            cache.queue_glyph(0, glyph.clone());
        }

        cache.cache_queued(|rect, data| unsafe {
            let mut rgb_data = Vec::with_capacity((4 * rect.width() * rect.height()) as usize);

            for pixel in data {
                rgb_data.push(r);
                rgb_data.push(g);
                rgb_data.push(b);
                rgb_data.push(*pixel);
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
        }).map_err(FontError::CacheWrite)?;

        //Get texture coordinates as Vector4f for each character
        let vec: Vec<CharacterPosition> = glyphs
            .iter()
            .filter_map(|glyph|
                match cache.rect_for(0, glyph).expect("Could not read cache.") {
                    Some((tex_pos, world_pos)) =>
                        Some(CharacterPosition {
                            texture_position: Vector4f::new(
                                tex_pos.min.x,
                                tex_pos.min.y,
                                tex_pos.width(),
                                tex_pos.height(),
                            ),
                            world_position: Vector2f::new(
                                world_pos.min.x as f32,
                                world_pos.min.y as f32
                            )
                        }),
                    None => None
                }
            ).collect();

        Ok(vec)
    }

    fn layout_paragraph(
        &self,
        text: &str,
        scale: Scale,
        width: u32,
    ) -> Vec<PositionedGlyph<'a>> {
        let font = &self.font;
        use unicode_normalization::UnicodeNormalization;
        let mut result = Vec::new();
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;
        for c in text.nfc() {
            if c.is_control() {
                match c {
                    '\r' => {
                        caret = point(0.0, caret.y + advance_height);
                    }
                    '\n' => {}
                    _ => {}
                }
                continue;
            }
            let base_glyph = font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, base_glyph.id());
            }
            last_glyph_id = Some(base_glyph.id());
            let mut glyph = base_glyph.scaled(scale).positioned(caret);
            if let Some(bb) = glyph.pixel_bounding_box() {
                if bb.max.x > width as i32 {
                    caret = point(0.0, caret.y + advance_height);
                    glyph = glyph.into_unpositioned().positioned(caret);
                    last_glyph_id = None;
                }
            }
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            result.push(glyph);
        }
        result
    }
}
