use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialFactors {
    pub base_color: [f32; 3],
    pub _padding1: f32,
    pub emissive: [f32; 3],
    pub _padding2: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha: f32,
    pub alpha_cutoff: f32,
}

impl MaterialFactors {
    pub fn new(
        base_color: [f32; 4],
        emissive: [f32; 3],
        metallic: f32,
        roughness: f32,
        alpha_cutoff: f32,
    ) -> Self {
        Self {
            base_color: cgmath::Vector4::from(base_color).truncate().into(),
            _padding1: 0.,
            emissive,
            _padding2: 0.,
            metallic,
            roughness,
            alpha: *(base_color.get(3).unwrap_or(&1.0)),
            alpha_cutoff,
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
        match alpha_mode {
            gltf::material::AlphaMode::Opaque => AlphaBlendMode::Opaque,
            gltf::material::AlphaMode::Mask => AlphaBlendMode::Mask,
            gltf::material::AlphaMode::Blend => AlphaBlendMode::Blend,
        }
    }
}

pub struct Material {
    pub pbr_material_factors_bind_group: wgpu::BindGroup,
    pub pbr_material_base_color_texture_bind_group: wgpu::BindGroup,
    pub factor_base_color: cgmath::Vector3<f32>,
    pub factor_emissive: cgmath::Vector3<f32>,
    pub factor_metallic: f32,
    pub factor_roughness: f32,
    pub factor_alpha: f32,
    pub factor_alpha_cutoff: f32,
    pub alpha_blend_mode: AlphaBlendMode,
    pub double_sided: bool,
}

impl Material {
    pub(crate) fn new(
        material: gltf::Material,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pbr_material_factors_bind_group_layout: &wgpu::BindGroupLayout,
        base_color_texture_bind_group_layout: &wgpu::BindGroupLayout,
        buffer_data: &Vec<Vec<u8>>,
    ) -> Self {
        let pbr_properties = material.pbr_metallic_roughness();

        // get base color texture
        let base_color_texture;
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
                    base_color_texture = crate::texture::Texture::from_bytes(
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
        } else {
            let bytes = include_bytes!("white.png");
            base_color_texture =
                crate::texture::Texture::from_bytes(device, queue, bytes, "default", None)
                    .expect("Couldn't load default texture");
        }

        // define the uniform
        let material_factors_uniform = MaterialFactors::new(
            pbr_properties.base_color_factor(),
            material.emissive_factor(),
            pbr_properties.metallic_factor(),
            pbr_properties.roughness_factor(),
            material.alpha_cutoff().unwrap_or(0.0),
        );

        // create the gpu bind group for material factors
        let pbr_mat_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PBR Material Buffer"),
            contents: bytemuck::cast_slice(&[material_factors_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let pbr_material_factors_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: pbr_material_factors_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pbr_mat_buffer.as_entire_binding(),
                }],
                label: None,
            });

        // create bind group for base color texture
        let pbr_material_base_color_texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &base_color_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &base_color_texture
                                // .expect("No base color texture found")
                                .view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &base_color_texture
                                // .expect("No base color texture found")
                                .sampler,
                        ),
                    },
                ],
                label: Some("base_color_texture_bind_group"),
            });

        // define this struct
        Self {
            pbr_material_factors_bind_group,
            pbr_material_base_color_texture_bind_group,
            factor_base_color: material_factors_uniform.base_color.into(),
            factor_emissive: material_factors_uniform.emissive.into(),
            factor_metallic: material_factors_uniform.metallic,
            factor_roughness: material_factors_uniform.roughness,
            factor_alpha: material_factors_uniform.alpha,
            factor_alpha_cutoff: material_factors_uniform.alpha_cutoff,
            alpha_blend_mode: AlphaBlendMode::from(material.alpha_mode()),
            double_sided: material.double_sided(),
        }
    }
}
