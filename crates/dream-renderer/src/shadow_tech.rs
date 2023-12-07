use wgpu::util::DeviceExt;
use wgpu::TextureFormat::Bgra8Unorm;

use dream_ecs::component::LightType;
use dream_math::{Matrix4, Point3, Vector3, Vector4};

use crate::camera::{Camera, CameraParams, CameraType};
use crate::camera_bones_light_bind_group::CameraBonesLightBindGroup;
use crate::instance::InstanceRaw;
use crate::lights::Lights;
use crate::model::{DrawModel, ModelVertex, Vertex};
use crate::pbr_material_tech::PbrMaterialTech;
use crate::render_storage::RenderStorage;
use crate::shader::Shader;
use crate::texture::Texture;

pub struct ShadowTech {
    pub shadow_cameras: Vec<Camera>,
    pub depth_textures: Vec<Texture>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: Option<wgpu::BindGroup>,
    pub frame_textures: Vec<Texture>,
    pub dummy_bind_group: wgpu::BindGroup,
    pub cascade_ends: Vec<f32>,
    pub cascade_settings_buffers: Vec<wgpu::Buffer>,
}

macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowCascadeSettingsUniform {
    cascade_end: f32,
    min_bias: f32,
    max_bias: f32,
    _padding0: f32,
    light_dir: [f32; 3],
    _padding1: f32,
}

impl ShadowTech {
    pub fn new(
        device: &wgpu::Device,
        camera_bones_lights_bind_group: &CameraBonesLightBindGroup,
        camera: &Camera,
        pbr_material_tech: &PbrMaterialTech,
    ) -> Self {
        let shader_write_shadow_buffer = Shader::new(
            device,
            include_str!("shader/shader_write_shadow_buffer.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_write_shadow_buffer"),
        );

        let texture_size = 4096;

        let shadow_cameras: Vec<Camera> = vec![];
        let depth_texture_0 = Texture::create_depth_texture(
            device,
            texture_size,
            texture_size,
            "shadow_depth_texture_0",
        );
        let depth_texture_1 = Texture::create_depth_texture(
            device,
            texture_size / 2,
            texture_size / 2,
            "shadow_depth_texture_1",
        );
        let depth_texture_2 = Texture::create_depth_texture(
            device,
            texture_size / 4,
            texture_size / 4,
            "shadow_depth_texture_2",
        );
        let depth_texture_3 = Texture::create_depth_texture(
            device,
            texture_size,
            texture_size,
            "shadow_depth_texture_3",
        );
        let depth_textures = vec![
            depth_texture_0,
            depth_texture_1,
            depth_texture_2,
            depth_texture_3,
        ];

        let frame_texture_0 = Texture::create_frame_texture(
            device,
            texture_size,
            texture_size,
            "shadow tech frame texture 0",
            Bgra8Unorm,
        );
        let frame_texture_1 = Texture::create_frame_texture(
            device,
            texture_size / 2,
            texture_size / 2,
            "shadow tech frame texture 1",
            Bgra8Unorm,
        );
        let frame_texture_2 = Texture::create_frame_texture(
            device,
            texture_size / 4,
            texture_size / 4,
            "shadow tech frame texture 2",
            Bgra8Unorm,
        );
        let frame_texture_3 = Texture::create_frame_texture(
            device,
            texture_size,
            texture_size,
            "shadow tech frame texture 3",
            Bgra8Unorm,
        );
        let frame_textures = vec![
            frame_texture_0,
            frame_texture_1,
            frame_texture_2,
            frame_texture_3,
        ];

        let cascade_ends = vec![
            camera.znear,
            camera.zfar / 300.0,
            camera.zfar / 80.0,
            camera.zfar / 20.0,
            camera.zfar,
        ];

        log::debug!("cascade ends {:?}", cascade_ends);

        // TODO: update light_dir via buffer update
        let cascade_settings_buffers = vec![
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shadow Cascade Settings Buffer 0"),
                contents: bytemuck::cast_slice(&[ShadowCascadeSettingsUniform {
                    cascade_end: cascade_ends[1],
                    min_bias: 0.000005,
                    max_bias: 0.005,
                    _padding0: 0.0,
                    light_dir: [-0.2, -0.4, -0.1],
                    _padding1: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shadow Cascade Settings Buffer 1"),
                contents: bytemuck::cast_slice(&[ShadowCascadeSettingsUniform {
                    cascade_end: cascade_ends[2],
                    min_bias: 0.0,
                    max_bias: 0.0,
                    _padding0: 0.0,
                    light_dir: [-0.2, -0.4, -0.1],
                    _padding1: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shadow Cascade Settings Buffer 2"),
                contents: bytemuck::cast_slice(&[ShadowCascadeSettingsUniform {
                    cascade_end: cascade_ends[3],
                    min_bias: 0.0,
                    max_bias: 0.0,
                    _padding0: 0.0,
                    light_dir: [-0.2, -0.4, -0.1],
                    _padding1: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shadow Cascade Settings Buffer 3"),
                contents: bytemuck::cast_slice(&[ShadowCascadeSettingsUniform {
                    cascade_end: cascade_ends[4],
                    min_bias: 0.0,
                    max_bias: 0.0,
                    _padding0: 0.0,
                    light_dir: [-0.2, -0.4, -0.1],
                    _padding1: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        ];

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Write Shadow Buffer Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bones_lights_bind_group.bind_group_layout,
                    &camera.camera_bind_group_layout,
                    &pbr_material_tech.pbr_material_textures_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline Write Shadow Buffer"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_write_shadow_buffer.get_shader_module(),
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
                // buffers: &[Vertex::desc()],
            },
            // fragment: None,
            fragment: Some(wgpu::FragmentState {
                module: shader_write_shadow_buffer.get_shader_module(),
                entry_point: "fs_main",
                targets: &[
                    // final deferred result render texture
                    Some(wgpu::ColorTargetState {
                        format: Bgra8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // cull_mode: Some(wgpu::Face::Front),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // shadow cascade 0
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), // wgpu::SamplerBindingType::Comparison
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // shadow cascade 1
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), // wgpu::SamplerBindingType::Comparison
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // shadow cascade 2
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), // wgpu::SamplerBindingType::Comparison
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // shadow cascade 3
                wgpu::BindGroupLayoutEntry {
                    binding: 12,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 13,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), // wgpu::SamplerBindingType::Comparison
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 14,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 15,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("shadow_tech_bind_group_layout"),
        });

        // shaders expect at least one bind group, so create one with no shadow
        let dummy_depth_texture =
            Texture::create_depth_texture(device, 4, 4, "dummy depth texture for shadows");
        let dummy_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                // shadow cascade 0
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_depth_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dummy_depth_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: Camera::new_orthographic(
                        &CameraParams {
                            eye: Point3::new(-999.0, -999.0, -999.0),
                            target: Point3::new(1000.0, 1000.0, 1000.0),
                            up: Vector3::new(0.0, 1.0, 0.0),
                            aspect: 1.5,
                            fovy: 1.0,
                            left: 0.0,
                            right: 1.0,
                            bottom: 0.0,
                            top: 1.0,
                            znear: 1.0,
                            zfar: 2.0,
                            camera_type: CameraType::Orthographic,
                        },
                        device,
                    )
                    .camera_buffer
                    .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: cascade_settings_buffers[0].as_entire_binding(),
                },
                // shadow cascade 1
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&dummy_depth_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&dummy_depth_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: Camera::new_orthographic(
                        &CameraParams {
                            eye: Point3::new(-999.0, -999.0, -999.0),
                            target: Point3::new(1000.0, 1000.0, 1000.0),
                            up: Vector3::new(0.0, 1.0, 0.0),
                            aspect: 1.5,
                            fovy: 1.0,
                            left: 0.0,
                            right: 1.0,
                            bottom: 0.0,
                            top: 1.0,
                            znear: 1.0,
                            zfar: 2.0,
                            camera_type: CameraType::Orthographic,
                        },
                        device,
                    )
                    .camera_buffer
                    .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: cascade_settings_buffers[1].as_entire_binding(),
                },
                // shadow cascade 2
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&dummy_depth_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&dummy_depth_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: Camera::new_orthographic(
                        &CameraParams {
                            eye: Point3::new(-999.0, -999.0, -999.0),
                            target: Point3::new(1000.0, 1000.0, 1000.0),
                            up: Vector3::new(0.0, 1.0, 0.0),
                            aspect: 1.5,
                            fovy: 1.0,
                            left: 0.0,
                            right: 1.0,
                            bottom: 0.0,
                            top: 1.0,
                            znear: 1.0,
                            zfar: 2.0,
                            camera_type: CameraType::Orthographic,
                        },
                        device,
                    )
                    .camera_buffer
                    .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: cascade_settings_buffers[2].as_entire_binding(),
                },
                // shadow cascade 3
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::TextureView(&dummy_depth_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: wgpu::BindingResource::Sampler(&dummy_depth_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: Camera::new_orthographic(
                        &CameraParams {
                            eye: Point3::new(-999.0, -999.0, -999.0),
                            target: Point3::new(1000.0, 1000.0, 1000.0),
                            up: Vector3::new(0.0, 1.0, 0.0),
                            aspect: 1.5,
                            fovy: 1.0,
                            left: 0.0,
                            right: 1.0,
                            bottom: 0.0,
                            top: 1.0,
                            znear: 1.0,
                            zfar: 2.0,
                            camera_type: CameraType::Orthographic,
                        },
                        device,
                    )
                    .camera_buffer
                    .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 15,
                    resource: cascade_settings_buffers[3].as_entire_binding(),
                },
            ],
            label: Some("shadow_tech_bind_group"),
        });

        Self {
            shadow_cameras,
            depth_textures,
            frame_textures,
            render_pipeline,
            bind_group_layout,
            bind_group: None,
            dummy_bind_group,
            cascade_ends,
            cascade_settings_buffers,
        }
    }

    pub fn render_shadow_depth_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        lights: &Lights,
        render_storage: &RenderStorage,
        camera_bones_lights_bind_group: &CameraBonesLightBindGroup,
        camera: &Camera,
    ) {
        for light in &lights.renderer_lights {
            if light.cast_shadow && light.light_type == LightType::DIRECTIONAL as u32 {
                // update shadow cascade uniforms
                let shadow_cascade_uniform_settings = vec![
                    ShadowCascadeSettingsUniform {
                        cascade_end: self.cascade_ends[1],
                        min_bias: 0.000005,
                        max_bias: 0.00087,
                        _padding0: 0.0,
                        light_dir: light.direction.into(),
                        _padding1: 0.0,
                    },
                    ShadowCascadeSettingsUniform {
                        cascade_end: self.cascade_ends[2],
                        min_bias: 0.0001,
                        max_bias: 0.001,
                        _padding0: 0.0,
                        light_dir: light.direction.into(),
                        _padding1: 0.0,
                    },
                    ShadowCascadeSettingsUniform {
                        cascade_end: self.cascade_ends[3],
                        min_bias: 0.00009,
                        max_bias: 0.0009,
                        _padding0: 0.0,
                        light_dir: light.direction.into(),
                        _padding1: 0.0,
                    },
                    ShadowCascadeSettingsUniform {
                        cascade_end: self.cascade_ends[4],
                        min_bias: 0.0003,
                        max_bias: 0.003,
                        _padding0: 0.0,
                        light_dir: light.direction.into(),
                        _padding1: 0.0,
                    },
                ];
                for cascade_idx in 0..4 {
                    // TODO: only write to buffer when hash changes for cascade_settings_buffers object (to hash just string concat floats and store in Option<String> variable)
                    queue.write_buffer(
                        &self.cascade_settings_buffers[cascade_idx],
                        0,
                        bytemuck::cast_slice(&[shadow_cascade_uniform_settings[cascade_idx]]),
                    );
                }

                // iterate and compute orthographic cameras for 4 cascades
                let mut camera_params = Vec::new();
                for i in 0..4 {
                    let proj = Matrix4::new_perspective(
                        camera.aspect,
                        camera.fovy,
                        self.cascade_ends[i],
                        self.cascade_ends[i + 1],
                    );

                    let cam_view_segment: Matrix4<f32> = camera.camera_uniform.view.into();
                    let mut frustum_corners: Vec<Vector4<f32>> = Vec::new();
                    let inv: Matrix4<f32> = (proj * cam_view_segment).try_inverse().unwrap();
                    for x in 0..2 {
                        for y in 0..2 {
                            for z in 0..2 {
                                let pt = inv
                                    * Vector4::new(
                                        2.0 * x as f32 - 1.0,
                                        2.0 * y as f32 - 1.0,
                                        2.0 * z as f32 - 1.0,
                                        1.0,
                                    );
                                frustum_corners.push(pt / pt.w);
                            }
                        }
                    }

                    let mut center: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
                    for corner in &frustum_corners {
                        center += corner.xyz();
                    }
                    center /= frustum_corners.len() as f32;

                    let eye = (center - light.direction.normalize()).into();
                    let target = (center).into();
                    let light_view =
                        Matrix4::look_at_rh(&eye, &target, &Vector3::new(0.0, 1.0, 0.0));

                    let mut min_x: f32 = f32::MAX;
                    let mut max_x: f32 = f32::MIN;
                    let mut min_y: f32 = f32::MAX;
                    let mut max_y: f32 = f32::MIN;
                    let mut min_z: f32 = f32::MAX;
                    let mut max_z: f32 = f32::MIN;

                    for v in &frustum_corners {
                        let trf = light_view * v;
                        min_x = min!(min_x, trf.x);
                        max_x = max!(max_x, trf.x);
                        min_y = min!(min_y, trf.y);
                        max_y = max!(max_y, trf.y);
                        min_z = min!(min_z, trf.z);
                        max_z = max!(max_z, trf.z);
                    }

                    // left, right, bottom, top, znear, zfar
                    let left = min_x;
                    let right = max_x;
                    let bottom = min_y;
                    let top = max_y;
                    let znear = -camera.zfar;
                    let zfar = camera.zfar;

                    // move view back a lot to capture entire scene
                    let eye = (center - camera.zfar / 2.0 * light.direction.normalize()).into();
                    let target = (center).into();

                    let camera_param = CameraParams {
                        eye,
                        target,
                        up: Vector3::new(0.0, 1.0, 0.0),
                        aspect: 1.5,
                        fovy: 0.0,
                        left,
                        right,
                        bottom,
                        top,
                        znear,
                        zfar,
                        camera_type: CameraType::Orthographic,
                    };

                    camera_params.push(camera_param);
                }

                if self.shadow_cameras.is_empty() {
                    self.shadow_cameras
                        .push(Camera::new_orthographic(&camera_params[0], device));
                    self.shadow_cameras
                        .push(Camera::new_orthographic(&camera_params[1], device));
                    self.shadow_cameras
                        .push(Camera::new_orthographic(&camera_params[2], device));
                    self.shadow_cameras
                        .push(Camera::new_orthographic(&camera_params[3], device));
                } else {
                    self.shadow_cameras
                        .get_mut(0)
                        .unwrap()
                        .update_ortho(&camera_params[0], queue);
                    self.shadow_cameras
                        .get_mut(1)
                        .unwrap()
                        .update_ortho(&camera_params[1], queue);
                    self.shadow_cameras
                        .get_mut(2)
                        .unwrap()
                        .update_ortho(&camera_params[2], queue);
                    self.shadow_cameras
                        .get_mut(3)
                        .unwrap()
                        .update_ortho(&camera_params[3], queue);
                }
            }
        }

        // TODO: remove any extra shadow cameras

        for (idx, shadow_camera) in self.shadow_cameras.iter().enumerate() {
            // define render pass
            let mut render_pass_write_shadow_buffer =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass Forward Rendering"),
                    // color_attachments: &[],
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.frame_textures[idx].view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_textures[idx].view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });
            render_pass_write_shadow_buffer.set_pipeline(&self.render_pipeline);

            // camera bind group
            render_pass_write_shadow_buffer.set_bind_group(
                0,
                &camera_bones_lights_bind_group.bind_group,
                &[],
            );

            render_pass_write_shadow_buffer.set_bind_group(
                1,
                &shadow_camera.camera_bind_group,
                &[],
            );

            // iterate through all meshes that should be instanced drawn
            for (render_map_key, transforms) in render_storage.render_map.iter() {
                let model_map = &render_storage.model_guids;
                // get the mesh to be instance drawn
                let model_guid = render_map_key.model_guid.clone();
                if model_map.get(&*model_guid).is_none() {
                    log::warn!("skipping drawing of model {model_guid}");
                    continue;
                }
                let model = model_map.get(&*model_guid).unwrap_or_else(|| {
                    panic!("no model loaded in renderer with guid {model_guid}")
                });
                let mesh_index = render_map_key.mesh_index;
                let mesh = model.meshes.get(mesh_index as usize).unwrap_or_else(|| {
                    panic!("no mesh at index {mesh_index} for model with guid {model_guid}")
                });
                // setup instancing buffer
                let instance_buffer = render_storage
                    .instance_buffer_map
                    .get(render_map_key)
                    .expect("No instance buffer found in map");
                render_pass_write_shadow_buffer.set_vertex_buffer(1, instance_buffer.slice(..));
                for primitive in &mesh.primitives {
                    // get the material and set it in the bind group
                    let material = model
                        .materials
                        .get(primitive.material)
                        .expect("No material at index");
                    // render all types of objects
                    if material.pbr_material_textures_bind_group.is_some() {
                        render_pass_write_shadow_buffer.set_bind_group(
                            2,
                            material.pbr_material_textures_bind_group.as_ref().unwrap(),
                            &[],
                        );
                        render_pass_write_shadow_buffer
                            .draw_primitive_instanced(&primitive, 0..transforms.len() as u32);
                    }
                }
            }
        }

        // update bind group
        if self.depth_textures.len() >= 4 && self.shadow_cameras.len() >= 4 {
            self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    // first shadow cascade
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.depth_textures[0].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.depth_textures[0].sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.shadow_cameras[0].camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.cascade_settings_buffers[0].as_entire_binding(),
                    },
                    // second shadow cascade
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&self.depth_textures[1].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&self.depth_textures[1].sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: self.shadow_cameras[1].camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: self.cascade_settings_buffers[1].as_entire_binding(),
                    },
                    // third shadow cascade
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(&self.depth_textures[2].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::Sampler(&self.depth_textures[2].sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: self.shadow_cameras[2].camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: self.cascade_settings_buffers[2].as_entire_binding(),
                    },
                    // fourth shadow cascade
                    wgpu::BindGroupEntry {
                        binding: 12,
                        resource: wgpu::BindingResource::TextureView(&self.depth_textures[3].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 13,
                        resource: wgpu::BindingResource::Sampler(&self.depth_textures[3].sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 14,
                        resource: self.shadow_cameras[3].camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 15,
                        resource: self.cascade_settings_buffers[3].as_entire_binding(),
                    },
                ],
                label: Some("shadow_tech_bind_group"),
            }));
        }
    }

    pub fn get_shadow_depth_textures(&self) -> &Vec<Texture> {
        &self.depth_textures
    }
}
