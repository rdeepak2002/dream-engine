use wgpu::util::DeviceExt;

use dream_math::{Matrix4, Point3, Vector3};

// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.0,
//     0.0, 0.0, 0.5, 1.0,
// );

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub(crate) camera_uniform: CameraUniform,
    pub(crate) camera_buffer: wgpu::Buffer,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl Camera {
    pub fn new(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
        device: &wgpu::Device,
    ) -> Self {
        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_proj(eye, target, up, aspect, fovy, znear, zfar);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        }
    }

    pub fn set_aspect_ratio(&mut self, queue: &wgpu::Queue, new_aspect_ratio: f32) {
        if self.aspect != new_aspect_ratio {
            self.aspect = new_aspect_ratio;
            self.camera_uniform.update_view_proj(
                self.eye,
                self.target,
                self.up,
                self.aspect,
                self.fovy,
                self.znear,
                self.zfar,
            );
            queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn update_view_proj(
        &mut self,
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) {
        let view = Matrix4::look_at_rh(&eye, &target, &up);
        let proj = Matrix4::new_perspective(aspect, fovy, znear, zfar);
        let view_proj = proj * view;
        self.view_proj = view_proj.into();
        self.inv_view_proj = view_proj
            .try_inverse()
            .expect("Unable to invert camera view projection matrix")
            .into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            inv_view_proj: Matrix4::identity().into(),
        }
    }
}
