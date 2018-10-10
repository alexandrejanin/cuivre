use self::{
    batches::{Batch, DrawCall},
    camera::Camera,
    mesh::{Mesh, MeshBuilder, Vertex},
    shaders::Program,
    shaders::{Shader, ShaderType},
    sprites::Sprite,
    text::{Font, TextSettings},
    textures::Texture,
};
use failure::Error;
use gl;
use maths::{Vector2f, Vector2u, Vector3f};
use sdl2;
use std::ptr;
use transform::Transform;

mod batches;
pub mod camera;
pub mod mesh;
pub mod shaders;
pub mod sprites;
pub mod text;
pub mod textures;

/// Error related to OpenGL drawing.
#[derive(Debug, Fail)]
pub enum GraphicsError {
    /// Error related to SDL.
    #[fail(display = "SDL Error: {}", _0)]
    SdlError(String),
    /// Error related to OpenGL.
    #[fail(display = "OpenGL Error: {}", _0)]
    GlError(String),
}

#[derive(Debug, Copy, Clone)]
pub struct WindowSettings<'a> {
    pub title: &'a str,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

/// Manages everything related to graphics and rendering.
pub struct GraphicsManager {
    window: sdl2::video::Window,

    #[allow(dead_code)]
    gl_context: sdl2::video::GLContext,

    /// Base shader program.
    program: Program,
    /// Base mesh used to draw sprites.
    quad: Mesh,

    /// All draw calls to be rendered this frame.
    batches: Vec<Batch>,
}

impl GraphicsManager {
    /// Initializes graphics from SDL object, resource loader, default shader paths and window settings
    pub fn new(sdl: &sdl2::Sdl, window_settings: WindowSettings) -> Result<Self, Error> {
        //Initialize VideoSubsystem
        let video = sdl.video().map_err(GraphicsError::SdlError)?;

        //Set OpenGL parameters
        {
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3);
        }

        //Create Window
        let window = video
            .window(
                window_settings.title,
                window_settings.width,
                window_settings.height,
            )
            .opengl()
            .resizable()
            .build()?;

        //Initialize OpenGL
        let gl_context = window.gl_create_context().map_err(GraphicsError::GlError)?;
        gl::load_with(|s| video.gl_get_proc_address(s) as *const gl::types::GLvoid);

        //Enable/disable vsync
        video.gl_set_swap_interval(if window_settings.vsync {
            sdl2::video::SwapInterval::VSync
        } else {
            sdl2::video::SwapInterval::Immediate
        });

        unsafe {
            //Depth testing
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);

            //Blending
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            //Clear color
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
        }

        //Load shaders
        let vertex_shader =
            Shader::from_source(include_str!("shaders/standard.vert"), ShaderType::Vertex)?;
        let fragment_shader =
            Shader::from_source(include_str!("shaders/standard.frag"), ShaderType::Fragment)?;
        let program = Program::from_shaders(vertex_shader, fragment_shader)?;

        //Build quad mesh
        let quad = MeshBuilder {
            vertices: vec![
                Vertex {
                    position: Vector3f::new(0.5, 0.5, 0.0),
                    uv: Vector2f::new(1.0, 0.0),
                },
                Vertex {
                    position: Vector3f::new(0.5, -0.5, 0.0),
                    uv: Vector2f::new(1.0, 1.0),
                },
                Vertex {
                    position: Vector3f::new(-0.5, -0.5, 0.0),
                    uv: Vector2f::new(0.0, 1.0),
                },
                Vertex {
                    position: Vector3f::new(-0.5, 0.5, 0.0),
                    uv: Vector2f::new(0.0, 0.0),
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
        .build();

        //Build and return graphics manager
        Ok(Self {
            window,
            gl_context,
            program,
            quad,
            batches: Vec::new(),
        })
    }

    /// Get the current window's size.
    pub fn window_size(&self) -> Vector2u {
        self.window.size().into()
    }

    /// Sets the OpenGL viewport. Call when the window is resized.
    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width as gl::types::GLint, height as gl::types::GLint);
        }
    }

    /// Draws a `Sprite` on a textured quad mesh.
    ///
    /// `transform` specifies the position, scale, and rotation
    /// of the drawn `Sprite`.
    ///
    /// `Camera` is the camera the `Sprite` is viewed from.
    ///
    /// Note: by default all sprites are square. For non-square sprites,
    /// you must use `transform.scale` to scale the quad appropriately.
    pub fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform, camera: &Camera) {
        let drawcall = DrawCall {
            program: self.program,
            mesh: self.quad,
            texture: sprite.texture(),
            tex_position: sprite.gl_position(),
            matrix: camera.matrix(self.window.size().into()) * transform.matrix(),
        };

        self.queue_drawcall(&drawcall);
    }

    /// Draws a string.
    pub fn draw_text(
        &mut self,
        text: &str,
        font: &mut Font,
        settings: TextSettings,
        transform: &Transform,
        camera: &Camera,
    ) -> Result<(), Error> {
        for char_position in font.get_glyphs(text, settings)? {
            let texture = font.texture();

            let char_transform = Transform {
                position: transform.position + Vector3f::new(
                    char_position.world_position.x,
                    char_position.world_position.y,
                    0.0,
                ),
                scale: Vector3f::new(
                    transform.scale.x * char_position.world_position.z,
                    transform.scale.y * char_position.world_position.w,
                    transform.scale.z,
                ),
                rotation: transform.rotation,
            };

            let drawcall = DrawCall {
                program: self.program,
                mesh: self.quad,
                texture,
                tex_position: char_position.texture_position,
                matrix: camera.matrix(self.window.size().into()) * char_transform.matrix(),
            };

            self.queue_drawcall(&drawcall);
        }

        Ok(())
    }

    /// Adds a drawcall to the render queue.
    ///
    /// If no suitable batch is found, a new one is created.
    pub fn queue_drawcall(&mut self, drawcall: &DrawCall) {
        for batch in &mut self.batches {
            //Attempts to add drawcall to batch
            if batch.add(drawcall) {
                return;
            }
        }

        //Could not find suitable batch, create a new one
        self.batches.push(Batch::new(drawcall));
    }

    /// Renders the current queued batches.
    pub fn render(&mut self) -> Result<(), Error> {
        //Clear render target
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        //println!("Rendering {} batches", self.batches.len());

        //Render batches
        for batch in &self.batches {
            self.draw(batch)?
        }

        //Clear queue
        self.batches.clear();

        //Swap buffers
        self.window.gl_swap_window();

        Ok(())
    }

    /// Draw a batch.
    fn draw(&self, batch: &Batch) -> Result<(), Error> {
        //Check that mesh is valid
        batch.mesh().check()?;

        //Use program
        batch.program().set_used();

        unsafe {
            //Bind texture
            gl::BindTexture(gl::TEXTURE_2D, batch.texture());

            //Bind mesh
            gl::BindVertexArray(batch.mesh().vao());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, batch.mesh().ebo());
        }

        //Bind objects data
        batch.buffer_data();

        //Draw batch
        unsafe {
            gl::DrawElementsInstanced(
                gl::TRIANGLES,                         //Draw mode
                batch.mesh().indices_count() as i32,   //Number of indices
                gl::UNSIGNED_INT,                      //Type of indices
                ptr::null(),                           //Starting index
                batch.obj_count() as gl::types::GLint, //Number of objects in batch
            );
        }

        Ok(())
    }
}
