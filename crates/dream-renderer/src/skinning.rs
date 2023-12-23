use wgpu::util::DeviceExt;

use dream_math::Matrix4;

pub struct SkinningTech {
    pub(crate) skinning_uniform: SkinningUniform,
    pub(crate) skinning_buffer: wgpu::Buffer,
}

impl SkinningTech {
    pub fn new(device: &wgpu::Device) -> Self {
        let skinning_uniform = SkinningUniform::default();

        let skinning_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skinning Buffer"),
            contents: bytemuck::cast_slice(&[skinning_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            skinning_uniform,
            skinning_buffer,
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
