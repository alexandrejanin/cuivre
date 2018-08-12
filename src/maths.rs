use cgmath;

//Point types
pub type Point2f = cgmath::Point2<f32>;
pub type Point3f = cgmath::Point3<f32>;

pub type Point2i = cgmath::Point2<i32>;
pub type Point3i = cgmath::Point3<i32>;

pub type Point2u = cgmath::Point2<u32>;
pub type Point3u = cgmath::Point3<u32>;

//Vector types

pub type Vector2f = cgmath::Vector2<f32>;
pub type Vector3f = cgmath::Vector3<f32>;
pub type Vector4f = cgmath::Vector4<f32>;

pub type Vector2i = cgmath::Vector2<i32>;
pub type Vector3i = cgmath::Vector3<i32>;

pub type Vector2u = cgmath::Vector2<u32>;
pub type Vector3u = cgmath::Vector3<u32>;

/// 4x4 column-major f32 matrix.
pub type Matrix4f = cgmath::Matrix4<f32>;

//Angles
pub type Deg = cgmath::Deg<f32>;
pub type Rad = cgmath::Rad<f32>;

//Euler
pub type Euler = cgmath::Euler<Deg>;

//Quaternion
pub type Quaternion = cgmath::Quaternion<f32>;
