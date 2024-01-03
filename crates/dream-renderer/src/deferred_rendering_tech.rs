use crate::camera_light_bind_group::CameraLightBindGroup;
use crate::instance::InstanceRaw;
use crate::material::Material;
use crate::model::{DrawModel, ModelVertex, Vertex};
use crate::pbr_material_tech::PbrMaterialTech;
use crate::render_storage::RenderStorage;
use crate::shader::Shader;
use crate::shadow_tech::ShadowTech;
use crate::texture;
use crate::texture::Texture;

pub struct DeferredRenderingTech {
    pub g_buffer_texture_views: [Option<texture::Texture>; 4],
    pub render_lights_for_deferred_gbuffers_bind_group: wgpu::BindGroup,
    pub render_pipeline_write_g_buffers: wgpu::RenderPipeline,
    pub render_pipeline_render_deferred_result: wgpu::RenderPipeline,
    pub render_lights_for_deferred_gbuffers_bind_group_layout: wgpu::BindGroupLayout,
}

impl DeferredRenderingTech {
    pub fn new(
        device: &wgpu::Device,
        target_texture_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        depth_texture: &Texture,
        pbr_material_tech: &PbrMaterialTech,
        shadow_tech: &ShadowTech,
        camera_bones_lights_bind_group: &CameraLightBindGroup,
    ) -> Self {
        let shader_write_g_buffers = Shader::new(
            device,
            include_str!("shader/shader_write_g_buffers.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_write_g_buffers"),
        );

        let shader_render_lights_for_deferred = Shader::new(
            device,
            include_str!("shader/shader_render_lights_for_deferred.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_render_lights_for_deferred"),
        );

        let g_buffer_texture_views = [
            Some(texture::Texture::create_frame_texture(
                &device,
                width,
                height,
                "Texture GBuffer Normal",
                wgpu::TextureFormat::Rgba16Float,
            )),
            Some(texture::Texture::create_frame_texture(
                &device,
                width,
                height,
                "Texture GBuffer Albedo",
                wgpu::TextureFormat::Bgra8Unorm,
            )),
            Some(texture::Texture::create_frame_texture(
                &device,
                width,
                height,
                "Texture GBuffer Emissive",
                wgpu::TextureFormat::Bgra8Unorm,
            )),
            Some(texture::Texture::create_frame_texture(
                &device,
                width,
                height,
                "Texture GBuffer AO Roughness Metallic",
                wgpu::TextureFormat::Bgra8Unorm,
            )),
        ];

        let render_lights_for_deferred_gbuffers_bind_group_layout = device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // normal
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
                    // albedo
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // emissive
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // ao roughness metallic
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // depth
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                ],
                label: Some("deferred_gbuffers_bind_group_layout"),
            });

        let render_lights_for_deferred_gbuffers_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_lights_for_deferred_gbuffers_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &g_buffer_texture_views[0].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &g_buffer_texture_views[1].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &g_buffer_texture_views[2].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(
                            &g_buffer_texture_views[3].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&depth_texture.view),
                    },
                ],
                label: Some("deferred_rendering_gbuffers_bind_group"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bones_lights_bind_group.bind_group_layout,
                    &pbr_material_tech.pbr_material_textures_bind_group_layout,
                    // &skinning_bind_group.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_write_g_buffers =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Write G Buffers"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_write_g_buffers.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_write_g_buffers.get_shader_module(),
                    entry_point: "fs_main",
                    targets: &[
                        // normal
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba16Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        // albedo
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        // emissive
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        // ao + roughness + metallic
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
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
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
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
        let quad_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Quad Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bones_lights_bind_group.bind_group_layout,
                    &render_lights_for_deferred_gbuffers_bind_group_layout,
                    &shadow_tech.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_render_deferred_result =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Render Deferred Result"),
                layout: Some(&quad_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_render_lights_for_deferred.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_render_lights_for_deferred.get_shader_module(),
                    entry_point: "fs_main",
                    targets: &[
                        // final deferred result render texture
                        Some(wgpu::ColorTargetState {
                            format: target_texture_format,
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
            g_buffer_texture_views,
            render_lights_for_deferred_gbuffers_bind_group_layout,
            render_lights_for_deferred_gbuffers_bind_group,
            render_pipeline_write_g_buffers,
            render_pipeline_render_deferred_result,
        }
    }

    pub fn render_to_gbuffers(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        depth_texture: &Texture,
        render_storage: &RenderStorage,
        camera_bones_lights_bind_group: &CameraLightBindGroup,
        filter_func: fn(&Material) -> bool,
    ) {
        // render to gbuffers
        // define render pass to write to GBuffers
        let mut render_pass_write_g_buffers =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass Write G Buffers"),
                color_attachments: &[
                    // albedo
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.g_buffer_texture_views[0].as_ref().unwrap().view,
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
                    }),
                    // normal
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.g_buffer_texture_views[1].as_ref().unwrap().view,
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
                    }),
                    // emissive
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.g_buffer_texture_views[2].as_ref().unwrap().view,
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
                    }),
                    // ao roughness metallic
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.g_buffer_texture_views[3].as_ref().unwrap().view,
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
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        render_pass_write_g_buffers.set_pipeline(&self.render_pipeline_write_g_buffers);

        // camera and lights bind group
        render_pass_write_g_buffers.set_bind_group(
            0,
            &camera_bones_lights_bind_group.bind_group,
            &[],
        );

        // camera and lights bind group
        // render_pass_write_g_buffers.set_bind_group(2, &skinning_bind_group.bind_group, &[]);

        // iterate through all meshes that should be instanced drawn
        for (render_map_key, transforms) in render_storage.render_map.iter() {
            let model_map = &render_storage.model_guids;
            // get the mesh to be instance drawn
            let model_guid = render_map_key.model_guid.clone();
            if model_map.get(&*model_guid).is_none() {
                log::warn!("skipping drawing of model {}", model_guid);
                continue;
            }
            let model = model_map
                .get(&*model_guid)
                .unwrap_or_else(|| panic!("no model loaded in renderer with guid {model_guid}"));
            let mesh_index = render_map_key.mesh_index;
            let mesh = model.meshes.get(mesh_index as usize).unwrap_or_else(|| {
                panic!("no mesh at index {mesh_index} for model with guid {model_guid}",)
            });
            // setup instancing buffer
            let instance_buffer = render_storage
                .instance_buffer_map
                .get(render_map_key)
                .expect("No instance buffer found in map");
            render_pass_write_g_buffers.set_vertex_buffer(1, instance_buffer.slice(..));
            for primitive in &mesh.primitives {
                // get the material and set it in the bind group
                let material = model
                    .materials
                    .get(primitive.material)
                    .expect("No material at index");
                if filter_func(material) && material.pbr_material_textures_bind_group.is_some() {
                    render_pass_write_g_buffers.set_bind_group(
                        1,
                        material.pbr_material_textures_bind_group.as_ref().unwrap(),
                        &[],
                    );
                    // draw the mesh
                    render_pass_write_g_buffers
                        .draw_primitive_instanced(&primitive, 0..transforms.len() as u32);
                }
            }
        }
    }

    pub fn combine_gbuffers_to_texture(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame_texture: &mut texture::Texture,
        depth_texture: &mut texture::Texture,
        shadow_tech: &ShadowTech,
        camera_bones_lights_bind_group: &CameraLightBindGroup,
    ) {
        // define render pass
        let mut render_pass_render_lights_for_deferred =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass Deferred Result"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        self.render_lights_for_deferred_gbuffers_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.render_lights_for_deferred_gbuffers_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.g_buffer_texture_views[0].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.g_buffer_texture_views[1].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &self.g_buffer_texture_views[2].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(
                            &self.g_buffer_texture_views[3].as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&depth_texture.view),
                    },
                ],
                label: Some("render_lights_for_deferred_gbuffers_bind_group"),
            });

        // camera and lights bind group
        render_pass_render_lights_for_deferred.set_bind_group(
            0,
            &camera_bones_lights_bind_group.bind_group,
            &[],
        );

        // gbuffers bind group
        render_pass_render_lights_for_deferred.set_bind_group(
            1,
            &self.render_lights_for_deferred_gbuffers_bind_group,
            &[],
        );

        // shadow tech bind group
        render_pass_render_lights_for_deferred.set_bind_group(
            2,
            shadow_tech
                .bind_group
                .as_ref()
                .unwrap_or(&shadow_tech.dummy_bind_group),
            &[],
        );

        render_pass_render_lights_for_deferred
            .set_pipeline(&self.render_pipeline_render_deferred_result);
        // draw quad (2 triangles defined by 6 vertices)
        render_pass_render_lights_for_deferred.draw(0..6, 0..1);
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // update gbuffers
        let texture_g_buffer_normal = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "Texture GBuffer Normal",
            wgpu::TextureFormat::Rgba16Float,
        );

        let texture_g_buffer_albedo = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "Texture GBuffer Albedo",
            wgpu::TextureFormat::Bgra8Unorm,
        );

        let texture_g_buffer_emissive = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "Texture GBuffer Emissive",
            wgpu::TextureFormat::Bgra8Unorm,
        );

        let texture_g_buffer_ao_roughness_metallic = texture::Texture::create_frame_texture(
            &device,
            width,
            height,
            "Texture GBuffer AO Roughness Metallic",
            wgpu::TextureFormat::Bgra8Unorm,
        );

        let g_buffer_texture_views = [
            Some(texture_g_buffer_normal),
            Some(texture_g_buffer_albedo),
            Some(texture_g_buffer_emissive),
            Some(texture_g_buffer_ao_roughness_metallic),
        ];

        self.g_buffer_texture_views = g_buffer_texture_views;
    }
}
