use wgpu::util::DeviceExt;
use wgpu::TextureFormat::Rgba32Float;
use wgpu::{Extent3d, ImageCopyTexture, Origin3d, TextureAspect, TextureFormat};

use dream_math::{pi, Matrix4, Point3, Vector3};

use crate::camera_light_bind_group::CameraLightBindGroup;
use crate::shader::Shader;
use crate::texture::Texture;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub inv_view: [[f32; 4]; 4],
    pub inv_projection: [[f32; 4]; 4],
}

pub struct CubemapTech {
    hdri_texture_bind_group: Option<wgpu::BindGroup>,
    render_pipeline_equirectangular_to_cubemap: wgpu::RenderPipeline,
    pub single_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_groups: Vec<wgpu::BindGroup>,
    pub cubemap_texture: Texture,
    pub irradiance_texture: Texture,
    pub cubemap_texture_bind_group_layout: wgpu::BindGroupLayout,
    cubemap_texture_bind_group: wgpu::BindGroup,
    hdri_to_cubemap_target_textures: Vec<Texture>,
    irradiance_cubemap_target_textures: Vec<Texture>,
    pub render_pipeline_draw_cubemap: wgpu::RenderPipeline,
    pub render_pipeline_irradiance_to_cubemap: wgpu::RenderPipeline,
    pub should_compute_irradiance_map: bool,
    pub irradiance_cubemap_texture_bind_group: wgpu::BindGroup,
    pub should_convert_hdr_to_cubemap: bool,
}

impl CubemapTech {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        camera_lights_bind_group: &CameraLightBindGroup,
    ) -> Self {
        let width = 1080;

        // define cubemap texture to render to
        let cubemap_texture = Texture::new_cubemap_texture(
            device,
            (width, width),
            Some("cubemap_texture"),
            Some(wgpu::TextureFormat::Rgba32Float),
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
        )
        .expect("Unable to create cubemap_texture");
        // define irradiance texture to render to
        let irradiance_texture = Texture::new_cubemap_texture(
            device,
            (width, width),
            Some("irradiance_texture"),
            Some(wgpu::TextureFormat::Rgba32Float),
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
        )
        .expect("Unable to create irradiance_texture");
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
        let shader_irradiance_convolution = Shader::new(
            device,
            include_str!("shader/irradiance_convolution.wgsl")
                .parse()
                .unwrap(),
            String::from("irradiance_conv"),
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
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    // sampler
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 1,
                    //     visibility: wgpu::ShaderStages::FRAGMENT,
                    //     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    //     count: None,
                    // },
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
        let mut hdri_to_cubemap_target_textures = Vec::new();
        let mut irradiance_cubemap_target_textures = Vec::new();
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
                inv_view: view.try_inverse().unwrap().into(),
                inv_projection: projection.try_inverse().unwrap().into(),
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
                Texture::create_frame_texture(&device, width, width, "target_texture", Rgba32Float);
            hdri_to_cubemap_target_textures.push(target_texture);

            let target_texture =
                Texture::create_frame_texture(&device, width, width, "target_texture", Rgba32Float);
            irradiance_cubemap_target_textures.push(target_texture);
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
        let irradiance_cubemap_texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &cubemap_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&irradiance_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&irradiance_texture.sampler),
                    },
                ],
                label: Some("irradiance_cubemap_texture_bind_group"),
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
        let render_pipelinelayout_irradiance_to_cubemap =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipelinelayout_irradiance_to_cubemap"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
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
                        format: TextureFormat::Rgba32Float,
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
                        // format: TextureFormat::Bgra8UnormSrgb,
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
        let render_pipeline_irradiance_to_cubemap =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("render_pipeline_irradiance_to_cubemap"),
                layout: Some(&render_pipelinelayout_irradiance_to_cubemap),
                vertex: wgpu::VertexState {
                    module: shader_irradiance_convolution.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_irradiance_convolution.get_shader_module(),
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba32Float,
                        // format: TextureFormat::Bgra8UnormSrgb,
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
            hdri_texture_bind_group: None,
            render_pipeline_equirectangular_to_cubemap,
            render_pipeline_draw_cubemap,
            single_texture_bind_group_layout,
            camera_bind_groups,
            cubemap_texture,
            cubemap_texture_bind_group_layout,
            cubemap_texture_bind_group,
            hdri_to_cubemap_target_textures,
            irradiance_cubemap_target_textures,
            irradiance_texture,
            render_pipeline_irradiance_to_cubemap,
            should_compute_irradiance_map: true,
            irradiance_cubemap_texture_bind_group,
            should_convert_hdr_to_cubemap: true,
        }
    }

    pub fn render_cubemap(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        hdr_frame_texture: &mut Texture,
        camera_lights_bind_group: &CameraLightBindGroup,
    ) {
        // once hdri image is loaded, create a texture on the gpu associated with it
        if self.hdri_texture_bind_group.is_none() {
            // load hdri image into texture
            // let hdri_image_bytes = include_bytes!("pure-sky.hdr");
            let hdri_image_bytes = include_bytes!("newport_loft.hdr");
            // let hdri_image_bytes = include_bytes!("puresky_2k.hdr");
            let (hdri_image_pixels, meta) = Texture::get_pixels_for_hdri_image(hdri_image_bytes);
            let hdri_texture = Texture::create_2d_texture(
                device,
                meta.width,
                meta.height,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                wgpu::FilterMode::Linear,
                None,
            );
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &hdri_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &bytemuck::cast_slice(&hdri_image_pixels),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        hdri_texture.texture.width() * std::mem::size_of::<[f32; 4]>() as u32,
                    ),
                    rows_per_image: Some(hdri_texture.texture.height()),
                },
                hdri_texture.texture.size(),
            );
            self.hdri_texture_bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.single_texture_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&hdri_texture.view),
                    }],
                    label: Some("hdri_texture_bind_group"),
                }));
        }

        if self.should_convert_hdr_to_cubemap {
            // draw equirectangular hdr map to cube
            for i in 0..6 {
                let mut render_pass_equirectangular_to_cubemap =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("render_pass_equirectangular_to_cubemap"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            // view: &no_hdr_frame_texture.view,
                            view: &self.hdri_to_cubemap_target_textures[i].view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                // load: wgpu::LoadOp::Load,
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
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
                    texture: &self.hdri_to_cubemap_target_textures[i].texture,
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
                    width: self.hdri_to_cubemap_target_textures[0].texture.width(),
                    height: self.hdri_to_cubemap_target_textures[0].texture.height(),
                    depth_or_array_layers: 1,
                };

                encoder.copy_texture_to_texture(source, destination, copy_size);
            }

            self.should_convert_hdr_to_cubemap = false;
        }

        // draw irradiance map to 6 textures
        if self.should_compute_irradiance_map {
            for i in 0..6 {
                let mut render_pass_irradiance_to_cubemap =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("render_pass_irradiance_to_cubemap"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            // view: &no_hdr_frame_texture.view,
                            view: &self.irradiance_cubemap_target_textures[i].view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                // load: wgpu::LoadOp::Load,
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                render_pass_irradiance_to_cubemap.set_bind_group(
                    0,
                    &self.camera_bind_groups[i],
                    &[],
                );
                render_pass_irradiance_to_cubemap.set_bind_group(
                    1,
                    &self.cubemap_texture_bind_group,
                    &[],
                );
                render_pass_irradiance_to_cubemap
                    .set_pipeline(&self.render_pipeline_irradiance_to_cubemap);
                render_pass_irradiance_to_cubemap.draw(0..36, 0..1);
            }
            self.should_compute_irradiance_map = false;

            // copy 6 rendered textures to a irradiance cubemap texture
            for i in 0..6 {
                let source = ImageCopyTexture {
                    texture: &self.irradiance_cubemap_target_textures[i].texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                };

                let destination = ImageCopyTexture {
                    texture: &self.irradiance_texture.texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: TextureAspect::All,
                };

                let copy_size = Extent3d {
                    width: self.irradiance_cubemap_target_textures[0].texture.width(),
                    height: self.irradiance_cubemap_target_textures[0].texture.height(),
                    depth_or_array_layers: 1,
                };

                encoder.copy_texture_to_texture(source, destination, copy_size);
            }
        }

        // draw cubemap
        {
            let mut render_pass_draw_cubemap =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render_pass_draw_cubemap"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        // view: &no_hdr_frame_texture.view,
                        view: &hdr_frame_texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            // load: wgpu::LoadOp::Load,
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            render_pass_draw_cubemap.set_bind_group(0, &camera_lights_bind_group.bind_group, &[]);
            render_pass_draw_cubemap.set_bind_group(1, &self.cubemap_texture_bind_group, &[]);
            // debug irradiance texture by displaying it as cubemap
            // render_pass_draw_cubemap.set_bind_group(
            //     1,
            //     &self.irradiance_cubemap_texture_bind_group,
            //     &[],
            // );
            render_pass_draw_cubemap.set_pipeline(&self.render_pipeline_draw_cubemap);
            render_pass_draw_cubemap.draw(0..36, 0..1);
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let width = 1080;
        self.cubemap_texture = Texture::new_cubemap_texture(
            &device,
            (width, width),
            Some("cubemap_texture"),
            Some(wgpu::TextureFormat::Rgba32Float),
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
        self.hdri_to_cubemap_target_textures = Vec::new();
        for _ in 0..6 {
            let target_texture =
                Texture::create_frame_texture(&device, width, width, "target_texture", Rgba32Float);
            self.hdri_to_cubemap_target_textures.push(target_texture);
        }
    }
}
