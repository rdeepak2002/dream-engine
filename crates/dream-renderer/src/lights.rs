use wgpu::util::DeviceExt;

use crate::renderer::RendererLight;

pub struct Lights {
    lights_uniform: LightsUniform,
    pub lights_bind_group_layout: wgpu::BindGroupLayout,
    pub lights_bind_group: wgpu::BindGroup,
}

impl Lights {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut lights_uniform = LightsUniform::default();

        let lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lights Buffer"),
            contents: bytemuck::cast_slice(&[lights_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let lights_bind_group_layout =
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
                label: Some("lights_bind_group_layout"),
            });

        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &lights_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_buffer.as_entire_binding(),
            }],
            label: Some("lights_bind_group"),
        });

        Self {
            lights_uniform,
            lights_bind_group_layout,
            lights_bind_group,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
    pub position: [f32; 3],
    pub _padding1: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightsUniform {
    lights: [LightData; 4],
}

impl LightsUniform {
    pub fn update_lights(&mut self, renderer_lights: Vec<RendererLight>) {
        todo!();
    }
}

impl Default for LightsUniform {
    fn default() -> Self {
        // TODO: set default to color 0, 0, 0
        Self {
            lights: [
                LightData {
                    position: [0., 0., 0.],
                    _padding1: 0,
                    color: [1., 1., 1.],
                    _padding2: 0,
                },
                LightData {
                    position: [0., 0., 0.],
                    _padding1: 0,
                    color: [0., 0., 0.],
                    _padding2: 0,
                },
                LightData {
                    position: [0., 0., 0.],
                    _padding1: 0,
                    color: [0., 0., 0.],
                    _padding2: 0,
                },
                LightData {
                    position: [0., 0., 0.],
                    _padding1: 0,
                    color: [0., 0., 0.],
                    _padding2: 0,
                },
            ],
        }
    }
}
