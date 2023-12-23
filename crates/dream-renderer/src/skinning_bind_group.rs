use crate::skinning::SkinningTech;

pub struct SkinningBindGroup {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl SkinningBindGroup {
    pub fn new(device: &wgpu::Device, skinning_tech: &SkinningTech) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("skinning_bind_group_layout"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: skinning_tech.skinning_buffer.as_entire_binding(),
            }],
            label: Some("skinning_bind_group"),
        });
        Self {
            bind_group_layout,
            bind_group,
        }
    }
}
