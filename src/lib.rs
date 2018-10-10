// Load extern crates
extern crate cgmath;
#[macro_use]
extern crate failure;
extern crate gl;
extern crate image;
extern crate rayon;
#[macro_use]
extern crate lazy_static;
extern crate rusttype;
extern crate sdl2;
extern crate unicode_normalization;

pub mod assets;
pub mod graphics;
pub mod input;
pub mod maths;
pub mod transform;

/// Initializes and returns an Sdl object, required to initialize some components such as GraphicsManager.
pub fn init_sdl() -> Result<sdl2::Sdl, String> {
    sdl2::init()
}
