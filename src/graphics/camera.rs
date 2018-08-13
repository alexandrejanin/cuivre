use cgmath::{self, Ortho, PerspectiveFov};
use maths::{Matrix4f, Point3f, Vector2f, Vector2u, Vector3f};

/// Different ways to calculate camera width and height from `size`.
pub enum CameraScaleMode {
    /// `size` will always be width/horizontal FOV.
    Width,
    /// `size` will always be height/vertical FOV.
    Height,
    /// `size` will be width if width < height, and vice versa.
    Min,
    /// `size` will be width if width > height, and vice versa.
    Max,
}

/// Represents a 3D projection and view.
///
/// Required for drawing sprites/meshes.
pub struct Camera {
    /// Position of the camera in world space.
    pub position: Point3f,
    /// Direction the camera is looking towards.
    pub direction: Vector3f,
    /// Near plane distance.
    pub near: f32,
    /// Far plane distance.
    pub far: f32,

    /// Size represents the frustum size if the camera is orthographic,
    ///
    /// and FOV if the camera is perspective.
    pub size: f32,
    /// Determines how the width and height of the camera will be calculated from `size` and window size.
    pub scale_mode: CameraScaleMode,

    /// Whether the camera is perspective or orthographic.
    pub perspective: bool,
}

impl Camera {
    /// Creates a new camera.
    ///
    /// Same as struct initialization.
    pub fn new(
        position: Point3f,
        direction: Vector3f,
        near: f32,
        far: f32,
        size: f32,
        scale_mode: CameraScaleMode,
        perspective: bool,
    ) -> Self {
        Self {
            position,
            direction,
            near,
            far,
            size,
            scale_mode,
            perspective,
        }
    }

    /// Make camera look at a point from its current position.
    pub fn look_at(&mut self, target: Point3f) {
        self.direction = target - self.position;
    }

    /// Combined projection and view matrices.
    pub fn matrix(&self, window_size: Vector2u) -> Matrix4f {
        self.proj_matrix(window_size) * self.view_matrix()
    }

    /// Projection matrix.
    pub fn proj_matrix(&self, window_size: Vector2u) -> Matrix4f {
        let ratio = window_size.x as f32 / window_size.y as f32;

        let size: Vector2f = match self.scale_mode {
            CameraScaleMode::Width => Vector2f::new(self.size, self.size / ratio),
            CameraScaleMode::Height => Vector2f::new(self.size * ratio, self.size),

            CameraScaleMode::Min => if ratio < 1.0 {
                Vector2f::new(self.size, self.size / ratio)
            } else {
                Vector2f::new(self.size * ratio, self.size)
            },

            CameraScaleMode::Max => if ratio > 1.0 {
                Vector2f::new(self.size, self.size / ratio)
            } else {
                Vector2f::new(self.size * ratio, self.size)
            },
        };

        if self.perspective {
            PerspectiveFov {
                fovy: cgmath::Deg(size.y).into(),
                near: self.near,
                far: self.far,
                aspect: size.x / size.y,
            }.into()
        } else {
            Ortho {
                left: -size.x / 2.0,
                right: size.x / 2.0,
                bottom: -size.y / 2.0,
                top: size.y / 2.0,
                near: self.near,
                far: self.far,
            }.into()
        }
    }

    /// View matrix.
    pub fn view_matrix(&self) -> Matrix4f {
        Matrix4f::look_at_dir(self.position, self.direction, Vector3f::new(0.0, 1.0, 0.0))
    }
}
