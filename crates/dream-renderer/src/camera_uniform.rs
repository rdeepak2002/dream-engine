use cgmath::SquareMatrix;

use crate::camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}
