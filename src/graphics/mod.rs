use self::{
    batches::{Batch, BatchList, DrawCall},
    camera::Camera,
    mesh::{Mesh, MeshBuilder, Vertex},
    shaders::Program,
    shaders::{Shader, ShaderType},
    sprites::Sprite,
    text::{Font, FontError},
    textures::Texture,
};
use gl;
use maths::{Vector2f, Vector2u, Vector3f};
use resources;
use sdl2;
use std::{error, fmt, ptr};
use transform::Transform;

mod batches;
pub mod camera;
pub mod mesh;
pub mod shaders;
pub mod sprites;
pub mod text;
pub mod textures;

/// Error related to OpenGL drawing.
#[derive(Debug)]
pub enum DrawingError {
    /// Error related to SDL.
    SdlError(String),
    /// Error related to OpenGL.
    GlError(String),
    /// Tried drawing a mesh that had no EBO set.
    MeshEBONotInitialized,
    /// Tried drawing a mesh that had no VAO set.
    MeshVAONotInitialized,
    /// Error related to reources handling.
    ResourceError(resources::ResourceError),
    /// Error related to OpenGL shaders.
    ShaderError(shaders::ShaderError),
    /// Error related to window building.
    WindowBuildError(sdl2::video::WindowBuildError),
}

impl fmt::Display for DrawingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DrawingError::SdlError(string) => write!(f, "{}", string),
            DrawingError::GlError(string) => write!(f, "{}", string),
            DrawingError::ResourceError(error) => write!(f, "{}", error),
            DrawingError::ShaderError(error) => write!(f, "{}", error),
            DrawingError::WindowBuildError(error) => write!(f, "{}", error),
            DrawingError::MeshEBONotInitialized => write!(f, "Mesh EBO not initialized"),
            DrawingError::MeshVAONotInitialized => write!(f, "Mesh VAO not initialized"),
        }
    }
}

impl error::Error for DrawingError {
    fn cause(&self) -> Option<&error::Error> {
        match self {
            DrawingError::ResourceError(error) => Some(error),
            DrawingError::ShaderError(error) => Some(error),
            DrawingError::WindowBuildError(error) => Some(error),

            _ => None,
        }
    }
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
    batches: BatchList,
}

impl GraphicsManager {
    /// Initializes graphics from SDL object, resource loader, default shader paths and window settings
    pub fn new(sdl: &sdl2::Sdl, window_settings: WindowSettings) -> Result<Self, DrawingError> {
        //Initialize VideoSubsystem
        let video = sdl.video().map_err(DrawingError::SdlError)?;

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
            .build()
            .map_err(DrawingError::WindowBuildError)?;

        //Initialize OpenGL
        let gl_context = window.gl_create_context().map_err(DrawingError::GlError)?;
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
            Shader::from_source(include_str!("shaders/standard.vert"), ShaderType::Vertex)
                .map_err(DrawingError::ShaderError)?;
        let fragment_shader =
            Shader::from_source(include_str!("shaders/standard.frag"), ShaderType::Fragment)
                .map_err(DrawingError::ShaderError)?;
        let program = Program::from_shaders(vertex_shader, fragment_shader)
            .map_err(DrawingError::ShaderError)?;

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
        }.build();

        //Build and return graphics manager
        Ok(Self {
            window,
            gl_context,
            program,
            quad,
            batches: BatchList::new(),
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

    /// Renders the current frame.
    pub fn render(&mut self) -> Result<(), DrawingError> {
        //Clear render target
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        //Render batches
        for batch in self.batches.iter() {
            self.draw(batch)?
        }

        //Clear queue
        self.batches.clear();

        //Swap buffers
        self.window.gl_swap_window();

        Ok(())
    }

    /// Add sprite to batch list.
    pub fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform, camera: &Camera) {
        self.batches.insert(&DrawCall {
            program: self.program,
            mesh: self.quad,
            texture: sprite.texture(),
            tex_position: sprite.gl_position(),
            matrix: camera.matrix(self.window.size().into()) * transform.matrix(),
        })
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font: &Font,
        transform: &Transform,
        camera: &Camera,
    ) -> Result<(), FontError> {
        let texture = font.texture();

        for tex_position in font.tex_positions(text)? {
            self.batches.insert(&DrawCall {
                program: self.program,
                mesh: self.quad,
                texture,
                tex_position,
                matrix: camera.matrix(self.window.size().into()) * transform.matrix(),
            })
        }

        Ok(())
    }

    /// Draw a batch.
    fn draw(&self, batch: &Batch) -> Result<(), DrawingError> {
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
