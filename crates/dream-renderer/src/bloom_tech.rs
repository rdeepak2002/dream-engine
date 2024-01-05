use std::ops::Div;

use wgpu::util::DeviceExt;
use wgpu::TextureFormat;

use dream_math::Vector2;

use crate::shader::Shader;
use crate::texture;

pub struct BloomMip {
    pub texture: texture::Texture,
    texture_bind_group: wgpu::BindGroup,
    mip_level_bind_group: wgpu::BindGroup,
}

pub struct BloomTech {
    pub mip_chain: Vec<BloomMip>,
    mip_chain_length: u32,
    pub render_pipeline_downsample: wgpu::RenderPipeline,
    pub render_pipeline_upsample: wgpu::RenderPipeline,
    pub bloom_mip_single_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub bloom_mip_info_bind_group_layout: wgpu::BindGroupLayout,
    pub frame_texture_bind_group: wgpu::BindGroup,
    pub filter_radius_bind_group: wgpu::BindGroup,
}

impl BloomTech {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        frame_texture: &texture::Texture,
    ) -> Self {
        // define shaders
        let shader_bloom_downsample = Shader::new(
            device,
            include_str!("shader/bloom_downsample.wgsl")
                .parse()
                .unwrap(),
            String::from("bloom_downsample"),
        );
        let shader_bloom_upsample = Shader::new(
            device,
            include_str!("shader/bloom_upsample.wgsl").parse().unwrap(),
            String::from("bloom_upsample"),
        );
        // define bind group layouts
        let bloom_mip_single_texture_bind_group_layout =
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
                label: Some("bloom_mip_single_texture_bind_group_layout"),
            });
        let bloom_mip_info_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("bloom_mip_info_bind_group_layout"),
            });
        let bloom_filter_radius_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("bloom_filter_radius_bind_group_layout"),
            });
        // define pipeline layouts
        let render_pipeline_layout_downsample =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bloom Tech Render Pipeline Layout"),
                bind_group_layouts: &[
                    &bloom_mip_single_texture_bind_group_layout,
                    &bloom_mip_info_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline_layout_upsample =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bloom Tech Render Pipeline Layout"),
                bind_group_layouts: &[
                    &bloom_mip_single_texture_bind_group_layout,
                    &bloom_filter_radius_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        // define render pipelines
        let render_pipeline_downsample =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Downsample"),
                layout: Some(&render_pipeline_layout_downsample),
                vertex: wgpu::VertexState {
                    module: shader_bloom_downsample.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_bloom_downsample.get_shader_module(),
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
        let render_pipeline_upsample =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Upsample"),
                layout: Some(&render_pipeline_layout_upsample),
                vertex: wgpu::VertexState {
                    module: shader_bloom_upsample.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_bloom_upsample.get_shader_module(),
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
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
        // generate mip chain of size n
        let mip_chain_length = 5;
        let mut mip_chain: Vec<BloomMip> = Vec::new();
        let mut mip_size = Vector2::<f32>::new(width as f32, height as f32);
        for i in 0..mip_chain_length {
            mip_size = mip_size.div(2.0);
            let texture = texture::Texture::create_frame_texture_with_filter(
                &device,
                mip_size.x as u32,
                mip_size.y as u32,
                format!("bloom_mip_chain_{i}").as_str(),
                TextureFormat::Rgba16Float,
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
            );
            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bloom_mip_single_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some("identify_bright_bind_group"),
            });
            let mip_level_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mip_level_buffer"),
                contents: bytemuck::cast_slice(&[i]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let mip_level_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("vertices buffer bind group"),
                layout: &bloom_mip_info_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: mip_level_buffer.as_entire_binding(),
                }],
            });
            mip_chain.push(BloomMip {
                texture,
                texture_bind_group,
                mip_level_bind_group,
            })
        }
        // bind group for first downsampling where we refer to frame texture
        let frame_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bloom_mip_single_texture_bind_group_layout,
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
            label: Some("frame_texture_bind_group"),
        });
        // filter radius bind group
        let filter_radius: f32 = 0.005;
        let filter_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("filter_radius_buffer"),
            contents: bytemuck::cast_slice(&[filter_radius]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let filter_radius_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("filter_radius_bind_group"),
            layout: &bloom_filter_radius_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: filter_radius_buffer.as_entire_binding(),
            }],
        });
        Self {
            mip_chain,
            mip_chain_length,
            render_pipeline_downsample,
            render_pipeline_upsample,
            bloom_mip_single_texture_bind_group_layout,
            bloom_mip_info_bind_group_layout,
            frame_texture_bind_group,
            filter_radius_bind_group,
        }
    }

    pub fn generate_bloom_texture(&self, encoder: &mut wgpu::CommandEncoder) {
        self.render_down_samples(encoder);
        self.render_up_samples(encoder);
    }

    pub fn render_down_samples(&self, encoder: &mut wgpu::CommandEncoder) {
        for i in 0..self.mip_chain_length {
            let mip = self.mip_chain.get(i as usize).unwrap();
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass_downsample"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip.texture.view,
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
            if i == 0 {
                // first down sample
                render_pass.set_bind_group(0, &self.frame_texture_bind_group, &[]);
            } else {
                // second to final down samples
                let previous_mip = self.mip_chain.get((i - 1) as usize).unwrap();
                render_pass.set_bind_group(0, &previous_mip.texture_bind_group, &[]);
            }
            render_pass.set_bind_group(1, &mip.mip_level_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline_downsample);
            render_pass.draw(0..6, 0..1);
        }
    }

    pub fn render_up_samples(&self, encoder: &mut wgpu::CommandEncoder) {
        for i in (1..self.mip_chain_length).rev() {
            let mip = self.mip_chain.get(i as usize).unwrap();
            let next_mip = self.mip_chain.get((i - 1) as usize).unwrap();
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass_upsample"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &next_mip.texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        // load: wgpu::LoadOp::Clear(wgpu::Color {
                        //     r: 0.0,
                        //     g: 0.0,
                        //     b: 0.0,
                        //     a: 0.0,
                        // }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_bind_group(0, &mip.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.filter_radius_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline_upsample);
            render_pass.draw(0..6, 0..1);
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // TODO: instead of duplicating code, move this to method
        let mut mip_chain: Vec<BloomMip> = Vec::new();
        let mut mip_size = Vector2::<f32>::new(width as f32, height as f32);
        for i in 0..self.mip_chain_length {
            mip_size = mip_size.div(2.0);
            let texture = texture::Texture::create_frame_texture_with_filter(
                &device,
                mip_size.x as u32,
                mip_size.y as u32,
                format!("bloom_mip_chain_{i}").as_str(),
                TextureFormat::Rgba16Float,
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
            );
            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bloom_mip_single_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some("identify_bright_bind_group"),
            });
            let mip_level_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mip_level_buffer"),
                contents: bytemuck::cast_slice(&[i]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let mip_level_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("vertices buffer bind group"),
                layout: &self.bloom_mip_info_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: mip_level_buffer.as_entire_binding(),
                }],
            });
            mip_chain.push(BloomMip {
                texture,
                texture_bind_group,
                mip_level_bind_group,
            })
        }
        self.mip_chain = mip_chain;
    }
}
