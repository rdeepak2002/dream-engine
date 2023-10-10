use crate::camera::Camera;
use crate::lights::Lights;
use crate::shader::Shader;

pub struct ShadowTech {
    pub shadow_cameras: Vec<Camera>,
}

impl ShadowTech {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader_write_shadow_buffer = Shader::new(
            device,
            include_str!("shader/shader_write_shadow_buffer.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_write_shadow_buffer"),
        );
        let shadow_cameras: Vec<Camera> = vec![]; // TODO: Camera::new_orthographic()
        Self { shadow_cameras }
    }

    pub fn render_shadow_depth_buffers(&mut self, lights: &Lights) {
        for light in &lights.renderer_lights {
            if light.cast_shadow {
                // log::debug!(
                //     "TODO: compute depth buffer stuff for this light for shadows {:?}",
                //     light
                // );
            }
        }
    }
}
