use gl;
use graphics::textures::{
    MaxFilterMode, MinFilterMode, Texture, TextureError, TextureOptions, WrapMode,
};
use maths::Vector4f;
use resources::Loadable;
use rusttype::{
    self,
    gpu_cache::{CacheBuilder, CacheReadErr, CacheWriteErr},
    point, PositionedGlyph, Scale,
};
use std::{error, fmt, os::raw::c_void};

const CACHE_WIDTH: u32 = 1024;
const CACHE_HEIGHT: u32 = 1024;

#[derive(Debug)]
pub enum FontError {
    CacheRead(CacheReadErr),
    CacheWrite(CacheWriteErr),
    Rusttype(rusttype::Error),
    Texture(TextureError),
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FontError::CacheRead(error) => write!(f, "{}", error),
            FontError::CacheWrite(error) => write!(f, "{}", error),
            FontError::Rusttype(error) => write!(f, "{}", error),
            FontError::Texture(error) => write!(f, "{}", error),
        }
    }
}

impl error::Error for FontError {}

pub struct Font {
    data: Vec<u8>,
    texture: Texture,
}

impl Loadable for Font {
    type LoadOptions = ();
    type LoadResult = Result<Self, FontError>;

    fn load(data: &[u8], _options: ()) -> Result<Self, FontError> {
        Self::from_bytes(data)
    }
}

impl Font {
    fn from_bytes(data: &[u8]) -> Result<Self, FontError> {
        //Cache texture settings
        let options = TextureOptions {
            h_wrap_mode: WrapMode::Repeat,
            v_wrap_mode: WrapMode::Repeat,
            min_filter_mode: MinFilterMode::Linear,
            max_filter_mode: MaxFilterMode::Linear,
        };

        let texture = Texture::from_bytes(
            &[0; (4 * CACHE_WIDTH * CACHE_HEIGHT) as usize],
            options,
            CACHE_WIDTH,
            CACHE_HEIGHT,
        ).map_err(FontError::Texture)?;

        Ok(Self {
            data: Vec::from(data),
            texture,
        })
    }

    pub fn font(&self) -> Result<rusttype::Font, FontError> {
        rusttype::Font::from_bytes(&self.data).map_err(FontError::Rusttype)
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Returns a sequence of the position of each character in the string on the cache.
    ///
    /// Can fail if the byte data for this font is invalid.
    pub fn tex_positions(&self, text: &str) -> Result<Vec<Vector4f>, FontError> {
        let font = self.font()?;
        let mut cache = CacheBuilder {
            width: CACHE_WIDTH as u32,
            height: CACHE_HEIGHT as u32,
            scale_tolerance: 0.01,
            position_tolerance: 0.01,
            pad_glyphs: true,
        }.build();

        let glyphs = Self::layout_paragraph(text, &font, Scale::uniform(10.0), 400);
        for glyph in &glyphs {
            cache.queue_glyph(0, glyph.clone());
        }

        //TODO upload glyphs to cache
        cache
            .cache_queued(|rect, data| unsafe {
                gl::TextureSubImage2D(
                    self.texture.id(),
                    0,
                    rect.min.x as i32,
                    rect.min.y as i32,
                    rect.width() as i32,
                    rect.height() as i32,
                    gl::RGBA,
                    gl::UNSIGNED_SHORT,
                    data.as_ptr() as *const c_void,
                );
            })
            .map_err(FontError::CacheWrite)?;

        let mut vec = Vec::with_capacity(text.len());

        //Get texture coordinates as Vector4f for each character
        for glyph in &glyphs {
            vec.push(
                cache
                    .rect_for(0, glyph)
                    .map(|coords| match coords {
                        Some((top_left, _)) => Vector4f::new(
                            top_left.min.x,
                            top_left.min.y,
                            top_left.width(),
                            top_left.height(),
                        ),
                        None => Vector4f::new(0.0, 0.0, 1.0, 1.0),
                    })
                    .map_err(FontError::CacheRead)?,
            );
        }

        Ok(vec)
    }

    fn layout_paragraph<'a>(
        text: &str,
        font: &'a rusttype::Font<'a>,
        scale: Scale,
        width: u32,
    ) -> Vec<PositionedGlyph<'a>> {
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
