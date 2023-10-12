use wgpu::util::DeviceExt;

use dream_math::Matrix4;

pub struct SkinningTech {
    pub(crate) skinning_uniform: SkinningUniform,
    pub(crate) skinning_buffer: wgpu::Buffer,
    // pub(crate) skinning_bind_group: wgpu::BindGroup,
    // pub(crate) skinning_bind_group_layout: wgpu::BindGroupLayout,
}

impl SkinningTech {
    pub fn new(device: &wgpu::Device) -> Self {
        let skinning_uniform = SkinningUniform::default();

        let skinning_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skinning Buffer"),
            contents: bytemuck::cast_slice(&[skinning_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // let skinning_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::all(),
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }],
        //         label: Some("skinning_bind_group_layout"),
        //     });
        //
        // let skinning_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &skinning_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: skinning_buffer.as_entire_binding(),
        //     }],
        //     label: Some("skinning_bind_group"),
        // });

        Self {
            skinning_uniform,
            skinning_buffer,
            // skinning_bind_group,
            // skinning_bind_group_layout,
        }
    }
}

impl SkinningTech {
    pub fn update_bone(&mut self, idx: u32, mat: Matrix4<f32>) {
        self.skinning_uniform.bone_transforms[idx as usize] = mat.into();
    }

    pub fn update_all_bones_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.skinning_buffer,
            0,
            bytemuck::cast_slice(&[self.skinning_uniform]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinningUniform {
    bone_transforms: [[[f32; 4]; 4]; 128],
}

impl Default for SkinningUniform {
    fn default() -> Self {
        Self {
            bone_transforms: [Matrix4::identity().into(); 128],
        }
    }
}
