use acro_math::{Float, Mat4};
use winit::dpi::PhysicalSize;

#[derive(Debug, Clone)]
pub struct Camera {
    pub(crate) camera_type: CameraType,
    pub(crate) projection_matrix: Mat4,
}

impl Camera {
    // TODO: Update projection matrix when window size changes
    pub fn new(camera_type: CameraType, window_width: u32, window_height: u32) -> Self {
        let projection_matrix = camera_type
            .create_projection_matrix(PhysicalSize::<u32>::new(window_width, window_height));
        Self {
            camera_type,
            projection_matrix,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CameraType {
    Perspective {
        fov: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    },
}

impl CameraType {
    pub fn create_projection_matrix(&self, window_size: PhysicalSize<u32>) -> Mat4 {
        let aspect = window_size.width as Float / window_size.height as Float;
        match *self {
            CameraType::Perspective { fov, near, far } => {
                Mat4::new_perspective(aspect, fov, near, far)
            }
            CameraType::Orthographic {
                left,
                right,
                top,
                bottom,
                near,
                far,
            } => Mat4::new_orthographic(left, right, top, bottom, near, far),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MainCamera;
