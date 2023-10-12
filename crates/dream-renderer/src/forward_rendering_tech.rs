use crate::camera_bones_light_bind_group::CameraBonesLightBindGroup;
use crate::instance::InstanceRaw;
use crate::material::Material;
use crate::model::{DrawModel, ModelVertex, Vertex};
use crate::pbr_material_tech::PbrMaterialTech;
use crate::render_storage::RenderStorage;
use crate::shader::Shader;
use crate::shadow_tech::ShadowTech;
use crate::texture;

pub struct ForwardRenderingTech {
    pub render_pipeline_forward_render_translucent_objects: wgpu::RenderPipeline,
}

impl ForwardRenderingTech {
    pub fn new(
        device: &wgpu::Device,
        target_texture_format: wgpu::TextureFormat,
        pbr_material_tech: &PbrMaterialTech,
        camera_bones_lights_bind_group: &CameraBonesLightBindGroup,
        shadow_tech: &ShadowTech,
    ) -> Self {
        let shader_forward_render = Shader::new(
            device,
            include_str!("shader/shader_forward.wgsl").parse().unwrap(),
            String::from("shader_forward_render"),
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Forward Rendering Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bones_lights_bind_group.bind_group_layout,
                    &pbr_material_tech.pbr_material_textures_bind_group_layout,
                    &shadow_tech.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_forward_render_translucent_objects =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Forward Rendering"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_forward_render.get_shader_module(),
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_forward_render.get_shader_module(),
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_texture_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
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

        Self {
            render_pipeline_forward_render_translucent_objects,
        }
    }

    pub fn render_to_output_texture(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_texture: &mut texture::Texture,
        depth_texture: &mut texture::Texture,
        render_storage: &RenderStorage,
        camera_bones_lights_bind_group: &CameraBonesLightBindGroup,
        shadow_tech: &ShadowTech,
        filter_func: fn(&Material) -> bool,
    ) {
        // define render pass
        let mut render_pass_forward_rendering =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass Forward Rendering"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        render_pass_forward_rendering
            .set_pipeline(&self.render_pipeline_forward_render_translucent_objects);

        // camera bind group
        render_pass_forward_rendering.set_bind_group(
            0,
            &camera_bones_lights_bind_group.bind_group,
            &[],
        );

        // shadow bind group
        render_pass_forward_rendering.set_bind_group(
            2,
            shadow_tech
                .bind_groups
                .get(0)
                .unwrap_or(&shadow_tech.dummy_bind_group),
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
            let model = model_map
                .get(&*model_guid)
                .unwrap_or_else(|| panic!("no model loaded in renderer with guid {model_guid}"));
            let mesh_index = render_map_key.mesh_index;
            let mesh = model.meshes.get(mesh_index as usize).unwrap_or_else(|| {
                panic!("no mesh at index {mesh_index} for model with guid {model_guid}")
            });
            // setup instancing buffer
            let instance_buffer = render_storage
                .instance_buffer_map
                .get(render_map_key)
                .expect("No instance buffer found in map");
            render_pass_forward_rendering.set_vertex_buffer(1, instance_buffer.slice(..));
            // get the material and set it in the bind group
            let material = model
                .materials
                .get(mesh.material)
                .expect("No material at index");
            // only draw transparent objects
            if filter_func(material) && material.pbr_material_textures_bind_group.is_some() {
                // render_pass_forward_rendering.set_bind_group(
                //     1,
                //     &material.pbr_material_factors_bind_group,
                //     &[],
                // );
                render_pass_forward_rendering.set_bind_group(
                    1,
                    material.pbr_material_textures_bind_group.as_ref().unwrap(),
                    &[],
                );
                // draw the mesh
                render_pass_forward_rendering.draw_mesh_instanced(mesh, 0..transforms.len() as u32);
            }
        }
    }
}
