use wgpu::util::DeviceExt;
use wgpu::TextureFormat::Rgba16Float;
use wgpu::{Extent3d, ImageCopyTexture, Origin3d, TextureAspect, TextureFormat};

use dream_math::{pi, Matrix4, Point3, Vector3};

use crate::camera_light_bind_group::CameraLightBindGroup;
use crate::image::Image;
use crate::shader::Shader;
use crate::texture::Texture;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

pub struct CubemapTech {
    hdri_image: Image,
    hdri_texture_bind_group: Option<wgpu::BindGroup>,
    render_pipeline_equirectangular_to_cubemap: wgpu::RenderPipeline,
    pub single_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_groups: Vec<wgpu::BindGroup>,
    pub cubemap_texture: Texture,
    cubemap_texture_bind_group_layout: wgpu::BindGroupLayout,
    cubemap_texture_bind_group: wgpu::BindGroup,
    target_textures: Vec<Texture>,
    pub render_pipeline_draw_cubemap: wgpu::RenderPipeline,
}

impl CubemapTech {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        camera_lights_bind_group: &CameraLightBindGroup,
    ) -> Self {
        // load hdri image into texture
        let hdr_image_bytes = include_bytes!("newport_loft.hdr");
        // let hdr_image_bytes = include_bytes!("puresky_2k.hdr");
        // TODO: correctly load hdri by referring to this: https://github.com/sotrh/learn-wgpu/blob/master/code/intermediate/tutorial13-hdr/src/resources.rs#L287
        let mut hdri_image = Image::default();
        hdri_image.load_from_bytes_threaded(
            hdr_image_bytes,
            "hdr_image_bytes",
            Some(String::from("image/hdr")),
        );
        // define cubemap texture to render to
        let cubemap_texture = Texture::new_cubemap_texture(
            device,
            (width, width),
            Some("cubemap_texture"),
            Some(wgpu::TextureFormat::Rgba16Float),
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
        )
        .expect("Unable to create cubemap_texture");
        // define shaders
        let shader_equirectangular_to_cubemap = Shader::new(
            device,
            include_str!("shader/equirectangular_to_cubemap.wgsl")
                .parse()
                .unwrap(),
            String::from("equirectangular_to_cubemap"),
        );
        let shader_cubemap = Shader::new(
            device,
            include_str!("shader/cubemap.wgsl").parse().unwrap(),
            String::from("cubemap"),
        );
        // define bind group layouts
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
        let single_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("single_texture_bind_group_layout"),
            });
        let cubemap_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    // sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("single_texture_bind_group_layout"),
            });
        // define bind groups
        let mut target_textures = Vec::new();
        let mut camera_bind_groups = Vec::new();
        let targets = vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(0.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
        ];
        let ups = vec![
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        ];
        for i in 0..6 {
            let view = Matrix4::look_at_rh(&Point3::new(0.0, 0.0, 0.0), &targets[i], &ups[i]);
            let projection = Matrix4::new_perspective(1.0, pi() / 2.0, 0.1, 10.0);
            let camera_uniform: CameraUniform = CameraUniform {
                view: view.into(),
                projection: projection.into(),
            };
            let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera_buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });
            camera_bind_groups.push(camera_bind_group);

            let target_texture =
                Texture::create_frame_texture(&device, width, width, "target_texture", Rgba16Float);

            target_textures.push(target_texture);
        }
        let cubemap_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &cubemap_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cubemap_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cubemap_texture.sampler),
                },
            ],
            label: Some("cubemap_texture_bind_group"),
        });
        // define pipeline layout
        let render_pipelinelayout_equirectangular_to_cubemap =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipelinelayout_equirectangular_to_cubemap"),
                bind_group_layouts: &[&camera_bind_group_layout, &single_texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipelinelayout_draw_cubemap =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipelinelayout_draw_cubemap"),
                bind_group_layouts: &[
                    &camera_lights_bind_group.bind_group_layout,
                    &cubemap_texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        // define render pipelines
        let render_pipeline_equirectangular_to_cubemap =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("render_pipeline_equirectangular_to_cubemap"),
                layout: Some(&render_pipelinelayout_equirectangular_to_cubemap),
                vertex: wgpu::VertexState {
                    module: shader_equirectangular_to_cubemap.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_equirectangular_to_cubemap.get_shader_module(),
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
        let render_pipeline_draw_cubemap =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("render_pipeline_draw_cubemap"),
                layout: Some(&render_pipelinelayout_draw_cubemap),
                vertex: wgpu::VertexState {
                    module: shader_cubemap.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_cubemap.get_shader_module(),
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
        Self {
            hdri_image,
            hdri_texture_bind_group: None,
            render_pipeline_equirectangular_to_cubemap,
            render_pipeline_draw_cubemap,
            single_texture_bind_group_layout,
            camera_bind_groups,
            cubemap_texture,
            cubemap_texture_bind_group_layout,
            cubemap_texture_bind_group,
            target_textures,
        }
    }

    pub fn render_cubemap(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        no_hdr_frame_texture: &mut Texture,
        camera_lights_bind_group: &CameraLightBindGroup,
    ) {
        // wait until hdri image is loaded on new thread
        if !self.hdri_image.loaded() {
            self.hdri_image.update();
            return;
        }

        // once hdri image is loaded, create a texture on the gpu associated with it
        if self.hdri_texture_bind_group.is_none() {
            // let rgba = self.hdri_image.to_rgba8();
            // let rgba = self.hdri_image.to_rgba8();
            let rgba = self.hdri_image.to_rgba8();
            let hdri_texture = Texture::new_with_filter(
                &device,
                &queue,
                rgba.to_vec(),
                rgba.dimensions(),
                Some("hdri_texture"),
                Some(wgpu::FilterMode::Linear),
                Some(TextureFormat::Rgba8Unorm),
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
            )
            .expect("Unable to create hdri texture from hdri image");
            self.hdri_texture_bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.single_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&hdri_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&hdri_texture.sampler),
                        },
                    ],
                    label: Some("hdri_texture_bind_group"),
                }));
        }

        for i in 0..6 {
            // draw equirectangular hdr map to cube
            let mut render_pass_equirectangular_to_cubemap =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render_pass_equirectangular_to_cubemap"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        // view: &no_hdr_frame_texture.view,
                        view: &self.target_textures[i].view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            // load: wgpu::LoadOp::Load,
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            render_pass_equirectangular_to_cubemap.set_bind_group(
                0,
                &self.camera_bind_groups[i],
                &[],
            );

            render_pass_equirectangular_to_cubemap.set_bind_group(
                1,
                self.hdri_texture_bind_group.as_ref().unwrap(),
                &[],
            );

            render_pass_equirectangular_to_cubemap
                .set_pipeline(&self.render_pipeline_equirectangular_to_cubemap);
            render_pass_equirectangular_to_cubemap.draw(0..36, 0..1);
        }

        // copy 6 rendered textures to a cubemap texture
        for i in 0..6 {
            let source = ImageCopyTexture {
                texture: &self.target_textures[i].texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            };

            let destination = ImageCopyTexture {
                texture: &self.cubemap_texture.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                aspect: TextureAspect::All,
            };

            let copy_size = Extent3d {
                width: self.target_textures[0].texture.width(),
                height: self.target_textures[0].texture.height(),
                depth_or_array_layers: 1,
            };

            encoder.copy_texture_to_texture(source, destination, copy_size);
        }

        // draw cubemap
        let mut render_pass_draw_cubemap = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass_draw_cubemap"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                // view: &no_hdr_frame_texture.view,
                view: &no_hdr_frame_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // load: wgpu::LoadOp::Load,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass_draw_cubemap.set_bind_group(0, &camera_lights_bind_group.bind_group, &[]);
        render_pass_draw_cubemap.set_bind_group(1, &self.cubemap_texture_bind_group, &[]);
        render_pass_draw_cubemap.set_pipeline(&self.render_pipeline_draw_cubemap);
        render_pass_draw_cubemap.draw(0..36, 0..1);
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.cubemap_texture = Texture::new_cubemap_texture(
            &device,
            (width, width),
            Some("cubemap_texture"),
            Some(wgpu::TextureFormat::Rgba16Float),
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
        )
        .expect("Unable to create cubemap_texture");
        self.cubemap_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.cubemap_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.cubemap_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.cubemap_texture.sampler),
                },
            ],
            label: Some("cubemap_texture_bind_group"),
        });
        self.target_textures = Vec::new();
        for _ in 0..6 {
            let target_texture =
                Texture::create_frame_texture(&device, width, width, "target_texture", Rgba16Float);
            self.target_textures.push(target_texture);
        }
    }
}
