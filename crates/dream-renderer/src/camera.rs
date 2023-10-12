use wgpu::util::DeviceExt;

use dream_math::{Matrix4, Point3, Quaternion, UnitQuaternion, Vector3};

// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.0,
//     0.0, 0.0, 0.5, 1.0,
// );

#[derive(Copy, Clone)]
pub enum CameraType {
    Perspective = 0,
    Orthographic = 1,
}

pub struct CameraParams {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub znear: f32,
    pub zfar: f32,
    pub camera_type: CameraType,
}

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub znear: f32,
    pub zfar: f32,
    camera_type: CameraType,
    pub(crate) camera_uniform: CameraUniform,
    pub(crate) camera_buffer: wgpu::Buffer,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl Camera {
    pub fn new_perspective(
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
        camera_uniform.update_view_proj_persp(eye, target, up, aspect, fovy, znear, zfar);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
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
            camera_type: CameraType::Perspective,
            camera_bind_group_layout,
            left: -10.0,
            right: 10.0,
            bottom: -10.0,
            top: 10.0,
        }
    }

    pub fn new_orthographic(camera_params: &CameraParams, device: &wgpu::Device) -> Self {
        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_proj_ortho(
            camera_params.eye,
            camera_params.target,
            camera_params.up,
            camera_params.left,
            camera_params.right,
            camera_params.bottom,
            camera_params.top,
            camera_params.znear,
            camera_params.zfar,
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
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
            eye: camera_params.eye,
            target: camera_params.target,
            up: camera_params.up,
            aspect: camera_params.aspect,
            fovy: std::f32::consts::FRAC_PI_4,
            znear: camera_params.znear,
            zfar: camera_params.zfar,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            camera_type: CameraType::Orthographic,
            left: camera_params.left,
            right: camera_params.right,
            bottom: camera_params.bottom,
            top: camera_params.top,
        }
    }

    pub fn update_ortho(&mut self, camera_params: &CameraParams, queue: &wgpu::Queue) {
        self.eye = camera_params.eye;
        self.target = camera_params.target;
        self.up = camera_params.up;
        self.left = camera_params.left;
        self.right = camera_params.right;
        self.bottom = camera_params.bottom;
        self.top = camera_params.top;
        self.znear = camera_params.znear;
        self.zfar = camera_params.zfar;
        self.camera_uniform.update_view_proj_ortho(
            camera_params.eye,
            camera_params.target,
            camera_params.up,
            camera_params.left,
            camera_params.right,
            camera_params.bottom,
            camera_params.top,
            camera_params.znear,
            camera_params.zfar,
        );
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn set_position_and_orientation(
        &mut self,
        queue: &wgpu::Queue,
        position: Point3<f32>,
        orientation: Quaternion<f32>,
    ) {
        self.eye = position;
        let forward_vector = UnitQuaternion::from_quaternion(orientation)
            .transform_vector(&Vector3::<f32>::new(0.0, 0.0, -1.0));
        self.target = self.eye + forward_vector;
        match self.camera_type {
            CameraType::Perspective => {
                self.camera_uniform.update_view_proj_persp(
                    self.eye,
                    self.target,
                    self.up,
                    self.aspect,
                    self.fovy,
                    self.znear,
                    self.zfar,
                );
            }
            CameraType::Orthographic => {
                // TODO: correctly update orthographic camera using orientation
                self.camera_uniform.update_view_proj_ortho(
                    self.eye,
                    self.target,
                    self.up,
                    self.left,
                    self.right,
                    self.bottom,
                    self.top,
                    self.znear,
                    self.zfar,
                );
            }
        }
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn set_aspect_ratio(&mut self, queue: &wgpu::Queue, new_aspect_ratio: f32) {
        if self.aspect != new_aspect_ratio {
            self.aspect = new_aspect_ratio;
            match self.camera_type {
                CameraType::Perspective => {
                    self.camera_uniform.update_view_proj_persp(
                        self.eye,
                        self.target,
                        self.up,
                        self.aspect,
                        self.fovy,
                        self.znear,
                        self.zfar,
                    );
                }
                CameraType::Orthographic => {}
            }
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
    position: [f32; 3],
    _padding: f32,
}

impl CameraUniform {
    pub fn update_view_proj_persp(
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
        self.position = eye.into();
    }

    pub fn update_view_proj_ortho(
        &mut self,
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    ) {
        let view = Matrix4::look_at_rh(&eye, &target, &up);
        let proj = Matrix4::new_orthographic(left, right, bottom, top, znear, zfar);
        let view_proj = proj * view;
        self.view_proj = view_proj.into();
        self.inv_view_proj = view_proj
            .try_inverse()
            .expect("Unable to invert camera view projection matrix")
            .into();
        self.position = eye.into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            inv_view_proj: Matrix4::identity().into(),
            position: [0.0, 0.0, 0.0],
            _padding: 1.,
        }
    }
}
