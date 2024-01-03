use wgpu::util::DeviceExt;

use dream_math::Matrix4;

use crate::render_storage::RenderStorage;
use crate::shader::Shader;

pub struct SkinningTech {
    pub(crate) skinning_uniform: SkinningUniform,
    pub(crate) skinning_buffer: wgpu::Buffer,
    pub skinning_bind_group: wgpu::BindGroup,
    pub skinning_compute_pipeline_layout: wgpu::PipelineLayout,
    pub vertices_bind_group_layout: wgpu::BindGroupLayout,
    pub primitive_info_bind_group_layout: wgpu::BindGroupLayout,
    pub skinning_compute_pipeline: wgpu::ComputePipeline,
}

impl SkinningTech {
    pub fn new(device: &wgpu::Device) -> Self {
        let skinning_uniform = SkinningUniform::default();

        let skinning_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skinning Buffer"),
            contents: bytemuck::cast_slice(&[skinning_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let skinning_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("skinning_bind_group_layout"),
            });

        let skinning_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &skinning_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: skinning_buffer.as_entire_binding(),
            }],
            label: Some("skinning_bind_group"),
        });

        let primitive_info_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("primitive_info_bind_group_layout"),
            });

        let vertices_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                    },
                }],
            });

        let skinning_compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("skinning compute shader pipeline layout"),
                bind_group_layouts: &[
                    &skinning_bind_group_layout,
                    &primitive_info_bind_group_layout,
                    &vertices_bind_group_layout,
                    &vertices_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let compute_shader_update_skinning_vertices = Shader::new(
            device,
            include_str!("shader/compute_shader_update_skinning_vertices.wgsl")
                .parse()
                .unwrap(),
            String::from("compute_shader_update_skinning_vertices"),
        );

        let skinning_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("compute pipeline"),
                layout: Some(&skinning_compute_pipeline_layout),
                module: compute_shader_update_skinning_vertices.get_shader_module(),
                entry_point: "cs_main",
            });

        Self {
            skinning_uniform,
            skinning_buffer,
            skinning_bind_group,
            skinning_compute_pipeline_layout,
            vertices_bind_group_layout,
            primitive_info_bind_group_layout,
            skinning_compute_pipeline,
        }
    }
}

impl SkinningTech {
    pub fn update_bone(&mut self, idx: u32, mat: Matrix4<f32>) {
        if (idx as usize) < self.skinning_uniform.bone_transforms.len() {
            self.skinning_uniform.bone_transforms[idx as usize] = mat.into();
        } else {
            log::warn!("Skipping bone since its index is out of bounds");
        }
    }

    pub fn update_all_bones_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.skinning_buffer,
            0,
            bytemuck::cast_slice(&[self.skinning_uniform]),
        );
    }

    pub fn compute_shader_update_vertices(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        render_storage: &mut RenderStorage,
    ) {
        // iterate through all meshes that should be instanced drawn
        for (render_map_key, transforms) in render_storage.render_map.iter() {
            let model_map = &mut render_storage.model_guids;
            // get the mesh to be instance drawn
            let model_guid = render_map_key.model_guid.clone();
            if model_map.get(&*model_guid).is_none() {
                log::warn!("skipping drawing of model {model_guid}");
                continue;
            }
            let model = model_map
                .get_mut(&*model_guid)
                .unwrap_or_else(|| panic!("no model loaded in renderer with guid {model_guid}"));
            let mesh_index = render_map_key.mesh_index;
            let mesh = model
                .meshes
                .get_mut(mesh_index as usize)
                .unwrap_or_else(|| {
                    panic!("no mesh at index {mesh_index} for model with guid {model_guid}")
                });
            for primitive in &mut mesh.primitives {
                // get the material and set it in the bind group
                let material = model
                    .materials
                    .get(primitive.material)
                    .expect("No material at index");
                // render all types of objects
                if material.pbr_material_textures_bind_group.is_some()
                    && primitive.skinned_vertex_buffer.is_some()
                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("compute skinning pass"),
                    });
                    cpass.set_bind_group(0, &self.skinning_bind_group, &[]);
                    cpass.set_bind_group(1, &primitive.primitive_info_bind_group, &[]);
                    cpass.set_bind_group(2, &primitive.vertex_buffer_bind_group, &[]);
                    cpass.set_bind_group(3, &primitive.skinned_vertices_buffer_bind_group, &[]);
                    cpass.set_pipeline(&self.skinning_compute_pipeline);
                    cpass.dispatch_workgroups(primitive.buffer_length / 64, 1, 1);
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinningUniform {
    bone_transforms: [[[f32; 4]; 4]; 256],
}

impl Default for SkinningUniform {
    fn default() -> Self {
        Self {
            bone_transforms: [Matrix4::identity().into(); 256],
        }
    }
}
