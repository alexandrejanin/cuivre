use gl;
use maths::{Vector2f, Vector2u, Vector3f};
use resources::{self, ResourceLoader};
use sdl2;
use std::{
    self,
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
    ptr,
};
use super::{
    batches::{Batch, BatchList, DrawCall},
    camera::Camera,
    mesh::{Mesh, MeshBuilder, Vertex},
    shaders::{self, Program},
    sprites::Sprite,
    Texture,
};
use transform::Transform;

///Error related to OpenGL drawing.
#[derive(Debug)]
pub enum DrawingError {
    ///Error related to SDL.
    SdlError(String),
    ///Error related to OpenGL.
    GlError(String),
    ///Error related to reources handling.
    ResourceError(resources::ResourceError),
    ///Error related to OpenGL shaders.
    ShaderError(shaders::ShaderError),
    ///Tried drawing a mesh that had no EBO set.
    MeshEBONotInitialized,
    ///Tried drawing a mesh that had no EBO set.
    MeshVAONotInitialized,
    ///Error related to window building.
    WindowBuildError(sdl2::video::WindowBuildError),
}

impl From<resources::ResourceError> for DrawingError {
    fn from(error: resources::ResourceError) -> Self {
        DrawingError::ResourceError(error)
    }
}

impl From<shaders::ShaderError> for DrawingError {
    fn from(error: shaders::ShaderError) -> Self {
        DrawingError::ShaderError(error)
    }
}

impl From<sdl2::video::WindowBuildError> for DrawingError {
    fn from(error: sdl2::video::WindowBuildError) -> Self {
        DrawingError::WindowBuildError(error)
    }
}

impl fmt::Display for DrawingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DrawingError::SdlError(string) => write!(f, "{}", string),
            DrawingError::GlError(string) => write!(f, "{}", string),
            DrawingError::ResourceError(error) => write!(f, "{}", error),
            DrawingError::ShaderError(error) => write!(f, "{}", error),
            DrawingError::MeshEBONotInitialized => write!(f, "Mesh EBO not initialized"),
            DrawingError::MeshVAONotInitialized => write!(f, "Mesh VAO not initialized"),
            DrawingError::WindowBuildError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for DrawingError {
    fn cause(&self) -> Option<&std::error::Error> {
        match self {
            DrawingError::ResourceError(error) => Some(error),
            DrawingError::ShaderError(error) => Some(error),
            DrawingError::WindowBuildError(error) => Some(error),

            _ => None,
        }
    }
}

/// Manages everything related to graphics and rendering.
pub struct GraphicsManager<'a> {
    resource_loader: &'a ResourceLoader,
    sdl: &'a sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    ///Basic shader program
    program: Program,
    ///Basic mesh used to draw sprites.
    quad: Mesh,
    ///Holds all textures that are loaded already.
    textures: HashMap<PathBuf, Texture>,
    ///All draw calls to be rendered this frame.
    batches: BatchList,
}

impl<'a> GraphicsManager<'a> {
    ///Initializes graphics from SDL object, resource loader, default shader paths and window settings
    pub fn new(
        sdl: &'a sdl2::Sdl,
        resource_loader: &'a ResourceLoader,
        vertex_shader_path: &Path,
        fragment_shader_path: &Path,
        title: &str,
        window_width: u32,
        window_height: u32,
        vsync: bool,
    ) -> Result<Self, DrawingError> {
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
            .window(title, window_width, window_height)
            .opengl()
            .resizable()
            .build()?;

        //Initialize OpenGL
        let gl_context = window.gl_create_context().map_err(DrawingError::GlError)?;
        gl::load_with(|s| video.gl_get_proc_address(s) as *const gl::types::GLvoid);

        //Enable/disable vsync
        video.gl_set_swap_interval(if vsync {
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
        let program =
            Program::load_shaders(resource_loader, vertex_shader_path, fragment_shader_path)?;

        //Build quad mesh
        let quad_builder = MeshBuilder {
            vertices: vec![
                Vertex {
                    position: Vector3f::new(0.5, 0.5, 0.0),
                    uv: Vector2f::new(1.0, 0.0),
                }, //Top right,
                Vertex {
                    position: Vector3f::new(0.5, -0.5, 0.0),
                    uv: Vector2f::new(1.0, 1.0),
                }, //Bottom right
                Vertex {
                    position: Vector3f::new(-0.5, -0.5, 0.0),
                    uv: Vector2f::new(0.0, 1.0),
                }, //Bottom left
                Vertex {
                    position: Vector3f::new(-0.5, 0.5, 0.0),
                    uv: Vector2f::new(0.0, 0.0),
                }, //Top left,
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };

        let quad = quad_builder.build();

        //Build and return graphics manager
        Ok(Self {
            resource_loader,
            sdl,
            video,
            window,
            gl_context,
            program,
            quad,
            textures: HashMap::new(),
            batches: BatchList::new(),
        })
    }

    ///Gets texture for given image, loading it if it wasn't loaded yet.
    pub fn get_texture(&mut self, path: &Path) -> Result<Texture, DrawingError> {
        //Return texture id if it's loaded already
        if let Some(texture) = self.textures.get(path) {
            return Ok(*texture);
        };

        //Texture wasn't found, load it
        let image = self.resource_loader.load_png(path)?;

        //Get image size
        let (width, height) = image.dimensions();

        //Allocate texture
        let mut texture_id = 0;

        unsafe {
            //Create texture
            gl::GenTextures(1, &mut texture_id);

            //Bind texture
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            //Fill texture
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as gl::types::GLint,
                width as gl::types::GLint,
                height as gl::types::GLint,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.as_ptr() as *const gl::types::GLvoid,
            );

            //Texture wrapping
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::REPEAT as gl::types::GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::REPEAT as gl::types::GLint,
            );

            //Texture filtering
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_NEAREST as gl::types::GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as gl::types::GLint,
            );

            //Generate mipmaps
            gl::GenerateMipmap(gl::TEXTURE_2D);

            //Unbind texture
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let texture = Texture {
            id: texture_id,
            size: Vector2u::new(width, height),
        };

        //Save texture so we don't have to load it again
        self.textures.insert(path.into(), texture);

        Ok(texture)
    }

    ///Get the current window's size.
    pub fn window_size(&self) -> Vector2u {
        self.window.size().into()
    }

    ///Sets the OpenGL viewport. Call when the window is resized.
    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width as gl::types::GLint, height as gl::types::GLint);
        }
    }

    ///Renders the current frame.
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

    ///Add sprite to batch list.
    pub fn draw_sprite(&mut self, sprite: Sprite, transform: Transform, camera: &Camera) {
        self.batches.insert(&DrawCall {
            program: self.program,
            mesh: self.quad,
            texture: *sprite.texture(),
            batch_vbo: self.quad.batch_vbo(),
            tex_position: sprite.gl_position(),
            matrix: camera.matrix(self.window.size().into()) * transform.matrix(),
        })
    }

    ///Draw a batch.
    fn draw(&self, batch: &Batch) -> Result<(), DrawingError> {
        //Check that mesh is valid
        batch.mesh().check()?;

        //Use program
        batch.program().set_used();

        unsafe {
            //Bind texture
            gl::BindTexture(gl::TEXTURE_2D, batch.texture().id());

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
                ptr::null(),         //Starting index
                batch.obj_count() as gl::types::GLint, //Number of objects in batch
            );
        }

        Ok(())
    }
}
