use wgpu::TextureFormat;

use crate::shader::Shader;
use crate::texture;

pub struct BloomTech {
    pub render_pipeline_bloom_tech: wgpu::RenderPipeline,
    pub render_pipeline_horizontal_blur: wgpu::RenderPipeline,
    pub render_pipeline_vertical_blur: wgpu::RenderPipeline,
    pub single_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub mask_texture: texture::Texture,
    pub blur_horizontal_texture: texture::Texture,
    pub blur_final_texture: texture::Texture,
    identify_bright_bind_group: Option<wgpu::BindGroup>,
    horizontal_blur_bind_group: Option<wgpu::BindGroup>,
    vertical_blur_bind_group: Option<wgpu::BindGroup>,
}

impl BloomTech {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let mask_texture = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "mask_texture",
            wgpu::TextureFormat::Rgba16Float,
        );

        let blur_horizontal_texture = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "blur_horizontal_texture",
            wgpu::TextureFormat::Rgba16Float,
        );

        let blur_final_texture = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "blur_final_texture",
            wgpu::TextureFormat::Rgba16Float,
        );

        let shader_identify_bright = Shader::new(
            device,
            include_str!("shader/shader_bloom_create_mask.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_bloom_identify_bright"),
        );

        let shader_blur_horizontal = Shader::new(
            device,
            include_str!("shader/shader_blur_horizontal.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_blur_horizontal"),
        );

        let shader_blur_vertical = Shader::new(
            device,
            include_str!("shader/shader_blur_vertical.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_blur_vertical"),
        );

        let single_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
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
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("deferred_gbuffers_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bloom Tech Render Pipeline Layout"),
                bind_group_layouts: &[&single_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_horizontal_blur =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Horizontal Blur"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_blur_horizontal.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_blur_horizontal.get_shader_module(),
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

        let render_pipeline_vertical_blur =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Vertical Blur"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_blur_vertical.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_blur_vertical.get_shader_module(),
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

        let render_pipeline_bloom_tech =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Forward Rendering"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_identify_bright.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_identify_bright.get_shader_module(),
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
            render_pipeline_bloom_tech,
            render_pipeline_horizontal_blur,
            render_pipeline_vertical_blur,
            single_texture_bind_group_layout,
            mask_texture,
            blur_horizontal_texture,
            blur_final_texture,
            identify_bright_bind_group: None,
            horizontal_blur_bind_group: None,
            vertical_blur_bind_group: None,
        }
    }

    pub fn generate_bright_mask(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        frame_texture: &mut texture::Texture,
    ) {
        let mut render_pass_identify_bright =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass_identify_bright"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mask_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
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

        self.identify_bright_bind_group =
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.single_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&frame_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&frame_texture.sampler),
                    },
                ],
                label: Some("identify_bright_bind_group"),
            }));

        render_pass_identify_bright.set_bind_group(
            0,
            self.identify_bright_bind_group.as_ref().unwrap(),
            &[],
        );
        render_pass_identify_bright.set_pipeline(&self.render_pipeline_bloom_tech);
        render_pass_identify_bright.draw(0..6, 0..1);
    }

    pub fn generate_blur(&mut self, encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device) {
        self.horizontal_blur(encoder, device, false);
        self.vertical_blur(encoder, device, false);
    }

    fn horizontal_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        clear_texture: bool,
    ) {
        let mut render_pass_horizontal_blur =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass_horizontal_blur"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blur_horizontal_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if clear_texture {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        self.horizontal_blur_bind_group =
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.single_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.mask_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.mask_texture.sampler),
                    },
                ],
                label: Some("horizontal_blur_bind_group"),
            }));

        render_pass_horizontal_blur.set_bind_group(
            0,
            self.horizontal_blur_bind_group.as_ref().unwrap(),
            &[],
        );
        render_pass_horizontal_blur.set_pipeline(&self.render_pipeline_horizontal_blur);
        render_pass_horizontal_blur.draw(0..6, 0..1);
    }

    fn vertical_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        clear_texture: bool,
    ) {
        let mut render_pass_vertical_blur =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass_vertical_blur"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blur_final_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if clear_texture {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        self.vertical_blur_bind_group =
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.single_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.blur_horizontal_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &self.blur_horizontal_texture.sampler,
                        ),
                    },
                ],
                label: Some("vertical_blur_bind_group"),
            }));

        render_pass_vertical_blur.set_bind_group(
            0,
            self.vertical_blur_bind_group.as_ref().unwrap(),
            &[],
        );
        render_pass_vertical_blur.set_pipeline(&self.render_pipeline_vertical_blur);
        render_pass_vertical_blur.draw(0..6, 0..1);
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let mask_texture = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "mask_texture",
            wgpu::TextureFormat::Rgba16Float,
        );

        let blur_horizontal_texture = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "blur_horizontal_texture",
            wgpu::TextureFormat::Rgba16Float,
        );

        let blur_final_texture = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "blur_final_texture",
            wgpu::TextureFormat::Rgba16Float,
        );

        self.mask_texture = mask_texture;
        self.blur_horizontal_texture = blur_horizontal_texture;
        self.blur_final_texture = blur_final_texture;
    }
}
