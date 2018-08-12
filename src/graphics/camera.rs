use cgmath::{self, Ortho, PerspectiveFov};
use maths::{Matrix4f, Point3f, Vector2f, Vector2u, Vector3f};

//Orthographic camera

/// Represents a 3D projection and view.
/// Required for drawing sprites/meshes.
pub struct Camera {
    ///Position of the camera in world space.
    pub position: Point3f,
    ///Direction the camera is looking towards.
    pub direction: Vector3f,
    ///Near plane distance.
    pub near: f32,
    ///Far plane distance.
    pub far: f32,

    ///Size if orthographic, FOV if perspective.
    size: Vector2f,

    ///Whether the camera is perspective or orthographic.
    pub perspective: bool,
}

impl Camera {
    /// Horizontal FOV if perspective camera, frustum width otherwise.
    pub fn width(&self) -> f32 { self.size.x }

    /// Vertical FOV if perspective camera, frustum height otherwise.
    pub fn height(&self) -> f32 { self.size.y }

    /// Set camera width while keeping aspect ratio.
    pub fn set_width(&mut self, width: f32) {
        self.size.y = width * self.size.y / self.size.x;
        self.size.x = width;
    }

    /// Set camera height while keeping aspect ratio.
    pub fn set_height(&mut self, height: f32) {
        self.size.x = height * self.size.x / self.size.y;
        self.size.y = height;
    }

    /// Creates a camera with given width.
    /// Height is calculated from window size.
    /// Width and height represent FOV if camera has perspective, frustum size otherwise.
    pub fn from_width(
        position: Point3f,
        direction: Vector3f,
        perspective: bool,
        near: f32,
        far: f32,
        width: f32,
        window_size: Vector2u,
    ) -> Self {
        let ratio = window_size.y as f32 / window_size.x as f32;
        let height = ratio * width;

        Self {
            position,
            direction,
            near,
            far,
            size: Vector2f::new(width, height),
            perspective,
        }
    }

    /// Creates a camera with given height.
    /// Width is calculated from window size.
    /// Width and height represent FOV if camera has perspective, frustum size otherwise.
    pub fn from_height(
        position: Point3f,
        direction: Vector3f,
        near: f32,
        far: f32,
        perspective: bool,
        height: f32,
        window_size: Vector2u,
    ) -> Self {
        let ratio = window_size.x as f32 / window_size.y as f32;
        let width = ratio * height;

        Self {
            position,
            direction,
            near,
            far,
            size: Vector2f::new(width, height),
            perspective,
        }
    }

    /// Resize the camera according to window size, conserving width and calculating a new height.
    pub fn resize_keep_width(&mut self, window_width: i32, window_height: i32) {
        self.size.y = self.size.x * window_height as f32 / window_width as f32;
    }

    /// Resize the camera according to window size, conserving height and calculating a new width.
    pub fn resize_keep_height(&mut self, window_width: i32, window_height: i32) {
        self.size.x = self.size.y * window_width as f32 / window_height as f32;
    }

    /// Make camera look at a point from its current position.
    pub fn look_at(&mut self, target: Point3f) {
        self.direction = target - self.position;
    }

    /// Combined projection and view matrices.
    pub fn matrix(&self) -> Matrix4f {
        self.proj_matrix() * self.view_matrix()
    }

    /// Projection matrix.
    pub fn proj_matrix(&self) -> Matrix4f {
        if self.perspective {
            PerspectiveFov {
                fovy: cgmath::Deg(self.size.y).into(),
                near: self.near,
                far: self.far,
                aspect: self.size.x / self.size.y,
            }.into()
        } else {
            Ortho {
                left: -self.size.x / 2.0,
                right: self.size.x / 2.0,
                bottom: -self.size.y / 2.0,
                top: self.size.y / 2.0,
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
