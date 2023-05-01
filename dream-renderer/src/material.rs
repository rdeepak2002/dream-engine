use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialFactors {
    pub base_color: [f32; 3],
    // pub metallic: f32,
    // pub roughness: f32,
    // pub emissive: [f32; 3],
    pub alpha: f32,
    // pub alpha_cutoff: f32,
}

impl MaterialFactors {
    pub fn new() -> Self {
        Self {
            base_color: [0., 0., 0.],
            // metallic: 0.0,
            // roughness: 0.0,
            // emissive: [0., 0., 0.],
            alpha: 1.0,
            // alpha_cutoff: 0.0,
        }
    }
}

pub enum AlphaBlendMode {
    Opaque = 0,
    Mask = 1,
    Blend = 2,
}

impl From<gltf::material::AlphaMode> for AlphaBlendMode {
    fn from(alpha_mode: gltf::material::AlphaMode) -> Self {
        return match alpha_mode {
            gltf::material::AlphaMode::Opaque => AlphaBlendMode::Opaque,
            gltf::material::AlphaMode::Mask => AlphaBlendMode::Mask,
            gltf::material::AlphaMode::Blend => AlphaBlendMode::Blend,
        };
    }
}

pub struct Material {
    pub bind_group: wgpu::BindGroup,
    pub factor_base_color: cgmath::Vector3<f32>,
    pub factor_alpha: f32,
    pub alpha_blend_mode: AlphaBlendMode,
    pub double_sided: bool,
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

        // define the uniform
        let material_factors_uniform = MaterialFactors {
            base_color: cgmath::Vector4::from(pbr_properties.base_color_factor())
                .truncate()
                .into(),
            // metallic: pbr_properties.metallic_factor(),
            // roughness: pbr_properties.roughness_factor(),
            // emissive: material.emissive_factor(),
            alpha: *(pbr_properties.base_color_factor().get(3).unwrap_or(&1.0)),
            // alpha_cutoff: material.alpha_cutoff().unwrap_or(0.0),
        };

        dbg!(material_factors_uniform);

        // create the gpu bind group for this
        let pbr_mat_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PBR Material Buffer"),
            contents: bytemuck::cast_slice(&[material_factors_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: pbr_mat_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // define this struct
        Self {
            bind_group,
            factor_base_color: material_factors_uniform.base_color.into(),
            factor_alpha: material_factors_uniform.alpha,
            alpha_blend_mode: AlphaBlendMode::from(material.alpha_mode()),
            double_sided: material.double_sided(),
        }
    }
}
