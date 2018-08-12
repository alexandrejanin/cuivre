use super::{
    mesh::{Mesh, BATCH_INSTANCE_SIZE, MAX_BATCH_SIZE},
    shaders::Program,
    Texture,
};
use gl;
use maths::{Matrix4f, Vector4f};
use std::{mem::size_of, slice::Iter};

#[derive(Debug)]
pub struct DrawCall {
    pub program: Program,
    pub mesh: Mesh,
    pub texture: Texture,
    pub batch_vbo: gl::types::GLuint,
    pub tex_position: Vector4f,
    pub matrix: Matrix4f,
}

///A queued draw call to be rendered.
pub struct Batch {
    ///Shader program to use to render.
    program: Program,
    ///Mesh to be rendered.
    mesh: Mesh,
    ///Texture to be rendered.
    texture: Texture,

    ///VBO containing transform matrices and texture info for each object
    batch_vbo: gl::types::GLuint,

    //TODO: replace with Vec?
    ///Stores the objects' info before it is passed to the VBO
    buffer: [f32; BATCH_INSTANCE_SIZE * MAX_BATCH_SIZE],

    ///Current amount of objects in the batch
    obj_count: usize,
}

impl Batch {
    pub fn program(&self) -> Program {
        self.program
    }
    pub fn mesh(&self) -> Mesh {
        self.mesh
    }
    pub fn texture(&self) -> Texture {
        self.texture
    }

    pub fn obj_count(&self) -> usize {
        self.obj_count
    }

    ///Creates an empty batch from specified drawcall.
    pub fn new(drawcall: &DrawCall) -> Self {
        let mut batch = Self {
            program: drawcall.program,
            mesh: drawcall.mesh,
            texture: drawcall.texture,
            batch_vbo: drawcall.batch_vbo,
            buffer: [0.0; BATCH_INSTANCE_SIZE * MAX_BATCH_SIZE],
            obj_count: 0,
        };

        batch.add(drawcall);

        batch
    }

    ///Adds an object to the batch. Returns false if the batch is full.
    pub fn add(&mut self, drawcall: &DrawCall) -> bool {
        if drawcall.program != self.program
            || drawcall.mesh != self.mesh
            || drawcall.texture != self.texture
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
            gl::BindBuffer(gl::ARRAY_BUFFER, self.batch_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<f32>() * self.obj_count * BATCH_INSTANCE_SIZE) as gl::types::GLsizeiptr,
                self.buffer.as_ptr() as *const gl::types::GLvoid,
                gl::STREAM_DRAW,
            );
        }
    }
}

///Contains and manages batches to be drawn.
pub struct BatchList {
    batches: Vec<Batch>,
}

impl BatchList {
    ///Initializes an empty BatchList.
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
        }
    }

    ///Adds a drawcall to the batch list, finding or creating a compatible batch.
    pub fn insert(&mut self, drawcall: &DrawCall) {
        for batch in &mut self.batches {
            //Attempts to add drawcall to batch
            if batch.add(drawcall) {
                return;
            }
        }

        //Could not find suitable batch, create a new one
        self.batches.push(Batch::new(drawcall));
    }

    pub fn clear(&mut self) {
        self.batches.clear()
    }

    pub fn iter(&self) -> Iter<Batch> {
        self.batches.iter()
    }
}
