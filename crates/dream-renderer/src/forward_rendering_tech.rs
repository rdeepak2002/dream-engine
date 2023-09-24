use crate::camera::Camera;
use crate::instance::InstanceRaw;
use crate::model::{DrawModel, ModelVertex, Vertex};
use crate::render_storage::RenderStorage;
use crate::texture;

pub struct ForwardRenderingTech {
    pub render_pipeline_forward_render_translucent_objects: wgpu::RenderPipeline,
}

impl ForwardRenderingTech {
    pub fn new(
        device: &wgpu::Device,
        render_pipeline_pbr_layout: &wgpu::PipelineLayout,
        target_texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader_forward_render = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Forward"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_forward.wgsl").into()),
        });

        let render_pipeline_forward_render_translucent_objects =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Forward Rendering"),
                layout: Some(&render_pipeline_pbr_layout),
                vertex: wgpu::VertexState {
                    module: &shader_forward_render,
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_forward_render,
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

    pub fn render_translucent_objects(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_texture: &mut texture::Texture,
        depth_texture: &mut texture::Texture,
        camera: &Camera,
        render_storage: &RenderStorage,
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
        render_pass_forward_rendering.set_bind_group(0, &camera.camera_bind_group, &[]);

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
            let is_translucent = material.factor_alpha < 1.0;
            if is_translucent && material.pbr_material_textures_bind_group.is_some() {
                render_pass_forward_rendering.set_bind_group(
                    1,
                    &material.pbr_material_factors_bind_group,
                    &[],
                );
                render_pass_forward_rendering.set_bind_group(
                    2,
                    material.pbr_material_textures_bind_group.as_ref().unwrap(),
                    &[],
                );
                // draw the mesh
                render_pass_forward_rendering.draw_mesh_instanced(mesh, 0..transforms.len() as u32);
            }
        }
    }
}
