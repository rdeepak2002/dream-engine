use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MaterialUniform {
    pub base_color: [f32; 4],
    // TODO: add support for base color texture
}

impl MaterialUniform {
    pub fn new() -> Self {
        Self {
            base_color: cgmath::Vector4::new(0., 0., 0., 1.).into(),
        }
    }
}

pub struct Material {
    pub base_color: cgmath::Vector4<f32>,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub(crate) fn new(
        material: gltf::Material,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        buffer_data: &Vec<Vec<u8>>,
    ) -> Self {
        let pbr_properties = material.pbr_metallic_roughness();

        // get base color texture
        if pbr_properties.base_color_texture().is_some() {
            let texture = pbr_properties
                .base_color_texture()
                .expect("No base color texture")
                .texture();
            let texture_name = texture.name().unwrap_or("No texture name");
            let texture_source = texture.source().source();
            match texture_source {
                gltf::image::Source::View { view, mime_type } => {
                    let parent_buffer_data = &buffer_data[view.buffer().index()];
                    let begin = view.offset();
                    let end = view.offset() + view.length();
                    let buf_dat = &parent_buffer_data[begin..end];
                    let mime_type = Some(mime_type.to_string());
                    let base_color_texture = crate::texture::Texture::from_bytes(
                        device,
                        queue,
                        buf_dat,
                        texture_name,
                        mime_type,
                    )
                    .expect("Couldn't load base color texture");
                }
                gltf::image::Source::Uri { uri, mime_type } => {
                    todo!();
                }
            };
        }

        // get base color factor
        let base_color = pbr_properties.base_color_factor();

        // create the uniform and the respective bind group for it
        let material_uniform = MaterialUniform { base_color };
        let pbr_mat_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PBR Buffer"),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: pbr_mat_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // define this struct
        Self {
            base_color: material_uniform.base_color.into(),
            bind_group,
        }
    }
}
