use gl;
use maths::{Vector2f, Vector3f};
use std::{cmp, mem, ptr};

//Max amount of instances in a batch
pub const MAX_BATCH_SIZE: usize = 1000;

//Size of 1 object in the VBO, in floats. 16: matrix, 4: tex coordinates
pub const BATCH_INSTANCE_SIZE: usize = 20;

#[derive(Debug, Fail)]
pub enum MeshError {
    /// Tried drawing a mesh that had no EBO set.
    #[fail(display = "Mesh EBO not initialized")]
    MeshEBONotInitialized,
    /// Tried drawing a mesh that had no VAO set.
    #[fail(display = "Mesh VAO not initialized")]
    MeshVAONotInitialized,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vector3f,
    pub uv: Vector2f,
}

impl Vertex {
    pub fn attrib_arrays() {
        let stride = mem::size_of::<Self>();

        unsafe {
            //Position
            Vertex::attrib_array(stride, 0, 0, 3);
            //Color
            Vertex::attrib_array(stride, 1, mem::size_of::<Vector3f>(), 2);
        }
    }

    unsafe fn attrib_array(stride: usize, location: gl::types::GLuint, offset: usize, length: i32) {
        //Vertex location 0: Position
        gl::EnableVertexAttribArray(location);
        gl::VertexAttribPointer(
            location, //Location
            length,   //Number of components per vertex
            gl::FLOAT,
            gl::FALSE,                          //Normalize
            stride as gl::types::GLint,         //Stride
            offset as *const gl::types::GLvoid, //Offset
        );
    }
}

///Represents a mesh that is loaded in OpenGL.
#[derive(Copy, Clone, Debug)]
pub struct Mesh {
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
    ebo: gl::types::GLuint,
    batch_vbo: gl::types::GLuint,
    vertex_count: usize,
    indices_count: usize,
}

impl Mesh {
    pub fn vbo(&self) -> gl::types::GLuint {
        self.vbo
    }
    pub fn vao(&self) -> gl::types::GLuint {
        self.vao
    }
    pub fn ebo(&self) -> gl::types::GLuint {
        self.ebo
    }
    pub fn batch_vbo(&self) -> gl::types::GLuint {
        self.batch_vbo
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    pub fn indices_count(&self) -> usize {
        self.indices_count
    }

    pub fn check(&self) -> Result<(), MeshError> {
        if self.ebo == 0 {
            return Err(MeshError::MeshEBONotInitialized);
        }
        if self.vao == 0 {
            return Err(MeshError::MeshVAONotInitialized);
        }

        Ok(())
    }
}

impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        self.ebo == other.ebo && self.vao == other.vao
    }
}

impl Eq for Mesh {}

impl PartialOrd for Mesh {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Mesh {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        //Sort by EBO, then VAO
        if self.ebo != other.ebo {
            self.ebo.cmp(&other.ebo)
        } else {
            self.vao.cmp(&other.vao)
        }
    }
}

#[derive(Debug)]
pub struct MeshBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<gl::types::GLuint>,
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBuilder {
    ///Initializes an empty MeshBuilder.
    pub fn new() -> MeshBuilder {
        MeshBuilder {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    ///Builds a mesh from current vertices and indices.
    pub fn build(&self) -> Mesh {
        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;
        let mut ebo: gl::types::GLuint = 0;

        unsafe {
            //Create objects
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            //Bind VAO
            gl::BindVertexArray(vao);

            //Bind and buffer VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, //Data length
                self.vertices.as_ptr() as *const gl::types::GLvoid, //Data location
                gl::STATIC_DRAW,
            );

            //Bind and buffer EBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr, //Data length
                self.indices.as_ptr() as *const gl::types::GLvoid, //Data location
                gl::STATIC_DRAW,
            );
        }

        //Fill VAO
        Vertex::attrib_arrays();

        //Create batch VBO
        let batch_vbo = Self::empty_vbo(MAX_BATCH_SIZE);
        Self::add_instanced_attribute(vao, batch_vbo, 2, 4, BATCH_INSTANCE_SIZE as i32, 0); //texture coordinates
        Self::add_instanced_attribute(vao, batch_vbo, 3, 4, BATCH_INSTANCE_SIZE as i32, 4); //1st column
        Self::add_instanced_attribute(vao, batch_vbo, 4, 4, BATCH_INSTANCE_SIZE as i32, 8); //2nd column
        Self::add_instanced_attribute(vao, batch_vbo, 5, 4, BATCH_INSTANCE_SIZE as i32, 12); //3rd column
        Self::add_instanced_attribute(vao, batch_vbo, 6, 4, BATCH_INSTANCE_SIZE as i32, 16); //4th column

        //Unbind everything
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        //Create mesh
        Mesh {
            vbo,
            vao,
            ebo,
            batch_vbo,
            vertex_count: self.vertices.len(),
            indices_count: self.indices.len(),
        }
    }

    ///Creates and returns an empty VBO than can fit 'floats' floats.
    fn empty_vbo(floats: usize) -> gl::types::GLuint {
        let mut vbo = 0;
        unsafe {
            //Create VBO
            gl::GenBuffers(1, &mut vbo);

            //Bind and allocate VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (floats * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                ptr::null(),
                gl::STREAM_DRAW,
            );

            //Unbind
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        vbo
    }

    fn add_instanced_attribute(
        vao: gl::types::GLuint,
        vbo: gl::types::GLuint,
        location: gl::types::GLuint,
        size: gl::types::GLint,
        stride: gl::types::GLsizei,
        offset: usize,
    ) {
        unsafe {
            //Bind and setup location
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindVertexArray(vao);
            gl::EnableVertexAttribArray(location);
            gl::VertexAttribPointer(
                location,
                size,
                gl::FLOAT,
                gl::FALSE,
                stride * mem::size_of::<f32>() as i32,
                (offset * mem::size_of::<f32>()) as *const gl::types::GLvoid,
            );
            gl::VertexAttribDivisor(location, 1);

            //Unbind
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }
}
