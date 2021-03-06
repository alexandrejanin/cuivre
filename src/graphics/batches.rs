use super::{
    mesh::{Mesh, BATCH_INSTANCE_SIZE, MAX_BATCH_SIZE},
    shaders::Program,
    textures::{Texture, TextureID},
};
use gl;
use maths::{Matrix4f, Vector4f};
use std::mem;

#[derive(Debug)]
pub struct DrawCall<'t> {
    pub program: Program,
    pub mesh: Mesh,
    pub texture: &'t Texture,
    pub tex_position: Vector4f,
    pub matrix: Matrix4f,
}

/// A queued draw call to be rendered.
pub struct Batch {
    /// Shader program to use to render.
    program: Program,
    /// Mesh to be rendered.
    mesh: Mesh,

    // TODO: use reference instead to get lifetime checking on the texture
    /// Texture to be rendered.
    texture: TextureID,

    /// Stores the objects' info before it is passed to the VBO
    buffer: [f32; BATCH_INSTANCE_SIZE * MAX_BATCH_SIZE],

    /// Current amount of objects in the batch
    obj_count: usize,
}

impl Batch {
    pub fn program(&self) -> Program {
        self.program
    }
    pub fn mesh(&self) -> Mesh {
        self.mesh
    }
    pub fn texture(&self) -> TextureID {
        self.texture
    }

    pub fn obj_count(&self) -> usize {
        self.obj_count
    }

    /// Creates an empty batch from specified drawcall.
    pub fn new(drawcall: &DrawCall) -> Self {
        let mut batch = Self {
            program: drawcall.program,
            mesh: drawcall.mesh,
            texture: drawcall.texture.id(),
            buffer: [0.0; BATCH_INSTANCE_SIZE * MAX_BATCH_SIZE],
            obj_count: 0,
        };

        batch.add(drawcall);

        batch
    }

    /// Adds an object to the batch. Returns false if the batch is full.
    pub fn add(&mut self, drawcall: &DrawCall) -> bool {
        if drawcall.program != self.program
            || drawcall.mesh != self.mesh
            || drawcall.texture.id() != self.texture
        {
            return false;
        }

        //Check if batch is full
        if self.obj_count >= MAX_BATCH_SIZE {
            return false;
        }

        let start_index = self.obj_count * BATCH_INSTANCE_SIZE;

        //Load tex position in buffer
        for i in 0..4 {
            self.buffer[start_index + i] = drawcall.tex_position[i];
        }

        //Load matrix in buffer
        for i in 0..16 {
            self.buffer[start_index + 4 + i] = drawcall.matrix[i / 4][i % 4];
        }

        self.obj_count += 1;

        true
    }

    pub fn buffer_data(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.mesh.batch_vbo());
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<f32>() * self.obj_count * BATCH_INSTANCE_SIZE)
                    as gl::types::GLsizeiptr,
                self.buffer.as_ptr() as *const gl::types::GLvoid,
                gl::STREAM_DRAW,
            );
        }
    }
}
