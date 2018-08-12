// Load extern crates
extern crate cgmath;
extern crate gl;
extern crate image;
extern crate ron;
extern crate sdl2;
extern crate serde;

pub use sdl2::event::{Event, WindowEvent};

pub mod graphics;
pub mod input;
pub mod maths;
pub mod resources;
pub mod transform;

/// Initializes and returns an Sdl object, required to initialize some components such as GraphicsManager.
pub fn init_sdl() -> Result<sdl2::Sdl, String> {
    sdl2::init()
}
