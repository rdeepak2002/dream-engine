use wgpu::TextureFormat::Bgra8Unorm;

use dream_math::Vector3;

use crate::camera::Camera;
use crate::instance::InstanceRaw;
use crate::lights::Lights;
use crate::model::{DrawModel, ModelVertex, Vertex};
use crate::render_storage::RenderStorage;
use crate::shader::Shader;
use crate::skinning::SkinningTech;
use crate::texture::Texture;

pub struct ShadowTech {
    pub shadow_cameras: Vec<Camera>,
    pub depth_textures: Vec<Texture>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub frame_textures: Vec<Texture>,
}

impl ShadowTech {
    pub fn new(device: &wgpu::Device, camera: &Camera, skinning_tech: &SkinningTech) -> Self {
        let shader_write_shadow_buffer = Shader::new(
            device,
            include_str!("shader/shader_write_shadow_buffer.wgsl")
                .parse()
                .unwrap(),
            String::from("shader_write_shadow_buffer"),
        );

        let shadow_cameras: Vec<Camera> = vec![];
        let depth_texture =
            Texture::create_depth_texture(device, 4096, 4096, "shadow_depth_texture_0");
        let depth_textures = vec![depth_texture];

        let frame_texture = Texture::create_frame_texture(
            device,
            4096,
            4096,
            "shadow tech frame texture",
            Bgra8Unorm,
        );
        let frame_textures = vec![frame_texture];

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Write Shadow Buffer Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.camera_bind_group_layout, // TODO: it's weird that we are passing in the camera bind group layout like this
                    &skinning_tech.skinning_bind_group_layout,
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
                // cull_mode: None,
                cull_mode: Some(wgpu::Face::Front),
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
                // depth texture
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
                // camera info like view projection matrix
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
            ],
            label: Some("shadow_tech_bind_group_layout"),
        });

        Self {
            shadow_cameras,
            depth_textures,
            frame_textures,
            render_pipeline,
            bind_group_layout,
            bind_groups: Vec::new(),
        }
    }

    pub fn render_shadow_depth_buffers(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        lights: &Lights,
        render_storage: &RenderStorage,
        skinning_tech: &SkinningTech,
    ) {
        self.shadow_cameras.clear();
        self.bind_groups.clear();

        // TODO: only clear the difference amount, and update the existing ones rather than creating new buffers every time this is called (which is every frame right now)
        for light in &lights.renderer_lights {
            // TODO: move these to enums so less refactoring (and be ale to easily convert from ECS light enum)
            let light_type_directional: u32 = 1;
            if light.cast_shadow && light.light_type == light_type_directional {
                let eye = light.position;
                let target = light.position + light.direction;
                let up = Vector3::new(0.0, 1.0, 0.0);
                let left = -10.0;
                let right = 10.0;
                let bottom = -10.0;
                let top = 10.0;
                let near_plane = 1.0;
                let far_plane = 7.5;
                // TODO: rather than creating a new buffer every frame, update an exising buffer at an index (while also removing extras)
                self.shadow_cameras.push(Camera::new_orthographic(
                    eye.into(),
                    target.into(),
                    up,
                    left,
                    right,
                    bottom,
                    top,
                    near_plane,
                    far_plane,
                    device,
                ));
            }
        }

        for (idx, shadow_camera) in self.shadow_cameras.iter().enumerate() {
            // update bind group so other
            // TODO: we'll need multiple bind groups in future

            // TODO: instead of clearing every time, just update the current bind group?
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.depth_textures[idx].view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.depth_textures[idx].sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: shadow_camera.camera_buffer.as_entire_binding(),
                    },
                ],
                label: Some("shadow_tech_bind_group"),
            });
            self.bind_groups.push(bind_group);

            // define render pass
            let mut render_pass_write_shadow_buffer =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass Forward Rendering"),
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
                &shadow_camera.camera_bind_group,
                &[],
            );

            // skinning bind group
            render_pass_write_shadow_buffer.set_bind_group(
                1,
                &skinning_tech.skinning_bind_group,
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
                // get the material and set it in the bind group
                let material = model
                    .materials
                    .get(mesh.material)
                    .expect("No material at index");
                // render all types of objects
                if material.pbr_material_textures_bind_group.is_some() {
                    // render_pass_write_shadow_buffer.set_bind_group(
                    //     2,
                    //     material.pbr_material_textures_bind_group.as_ref().unwrap(),
                    //     &[],
                    // );
                    render_pass_write_shadow_buffer
                        .draw_mesh_instanced(mesh, 0..transforms.len() as u32);
                }
            }
        }
    }

    pub fn get_shadow_depth_textures(&self) -> &Vec<Texture> {
        &self.depth_textures
    }
}
