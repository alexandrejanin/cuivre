use cgmath;
use maths::{Euler, Matrix4f, Point3f, Quaternion, Vector3f};

///Represents a transformation in 3D space: translation (position), scale, rotation (Euler's angles).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Point3f,
    pub scale: Vector3f,
    pub rotation: Vector3f,
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    ///Create a transform with default position, scale, and rotation.
    pub fn new() -> Self {
        Self {
            position: Point3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(1.0, 1.0, 1.0),
            rotation: Vector3f::new(0.0, 0.0, 0.0),
        }
    }

    ///Create a transform with set position, with default scale and rotation.
    pub fn from_position(position: Point3f) -> Self {
        Self {
            position,
            scale: Vector3f::new(1.0, 1.0, 1.0),
            rotation: Vector3f::new(0.0, 0.0, 0.0),
        }
    }

    ///Create a transform with set scale, with default position and rotation.
    pub fn from_scale(scale: Vector3f) -> Self {
        Self {
            position: Point3f::new(0.0, 0.0, 0.0),
            scale,
            rotation: Vector3f::new(0.0, 0.0, 0.0),
        }
    }

    ///Create a transform with set rotation, with default position and scale.
    pub fn from_rotation(rotation: Vector3f) -> Self {
        Self {
            position: Point3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(1.0, 1.0, 1.0),
            rotation,
        }
    }

    ///Creates a matrix that applies the transform to a vector, or another matrix.
    pub fn matrix(&self) -> Matrix4f {
        let quaternion = Quaternion::from(Euler::new(
            cgmath::Deg(self.rotation.x),
            cgmath::Deg(self.rotation.y),
            cgmath::Deg(self.rotation.z),
        ));

        let rot = Matrix4f::from(quaternion);
        let scale = Matrix4f::new(
            self.scale.x,
            0.,
            0.,
            0.,
            0.,
            self.scale.y,
            0.,
            0.,
            0.,
            0.,
            self.scale.z,
            0.,
            0.,
            0.,
            0.,
            1.,
        );
        let translate = Matrix4f::new(
            1.,
            0.,
            0.,
            0.,
            0.,
            1.,
            0.,
            0.,
            0.,
            0.,
            1.,
            0.,
            self.position.x,
            self.position.y,
            self.position.z,
            1.,
        );

        translate * rot * scale
    }
}
