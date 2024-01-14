use std::path::Path;
use wgpu::util::DeviceExt;

use crate::image::Image;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialFactors {
    pub base_color: [f32; 3],
    pub _padding1: f32,
    pub emissive: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub alpha: f32,
    pub alpha_cutoff: f32,
}

impl MaterialFactors {
    pub fn new(
        base_color: [f32; 4],
        emissive: [f32; 4],
        metallic: f32,
        roughness: f32,
        alpha_cutoff: f32,
    ) -> Self {
        Self {
            base_color: [
                *base_color.get(0).unwrap(),
                *base_color.get(1).unwrap(),
                *base_color.get(2).unwrap(),
            ],
            _padding1: 0.,
            emissive,
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
    // pub pbr_material_factors_bind_group: wgpu::BindGroup,
    pub pbr_material_textures_bind_group: Option<wgpu::BindGroup>,
    pub factor_base_color: dream_math::Vector3<f32>,
    pub factor_emissive: dream_math::Vector4<f32>,
    pub factor_metallic: f32,
    pub factor_roughness: f32,
    pub factor_alpha: f32,
    pub factor_alpha_cutoff: f32,
    pub alpha_blend_mode: AlphaBlendMode,
    pub double_sided: bool,
    pub base_color_image: Image,
    pub metallic_roughness_image: Image,
    pub normal_map_image: Image,
    pub emissive_image: Image,
    pub occlusion_image: Image,
    pub pbr_mat_buffer: wgpu::Buffer,
}

impl Material {
    pub(crate) fn new<'a>(
        material: gltf::Material<'a>,
        device: &wgpu::Device,
        pbr_material_factors_bind_group_layout: &wgpu::BindGroupLayout,
        buffer_data: &[Vec<u8>],
        image_folder: &Path,
    ) -> Self {
        let pbr_properties = material.pbr_metallic_roughness();

        // get base color texture
        let mut base_color_image = Image::default();
        match pbr_properties.base_color_texture() {
            None => {
                let bytes = include_bytes!("white.png");
                base_color_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                base_color_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
            }
        }

        // get metallic texture
        let mut metallic_roughness_image = Image::default();
        match pbr_properties.metallic_roughness_texture() {
            None => {
                let bytes = include_bytes!("black.png");
                metallic_roughness_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                metallic_roughness_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
            }
        }

        // get normal map texture
        let mut normal_map_image = Image::default();
        match material.normal_texture() {
            None => {
                let bytes = include_bytes!("default_normal.png");
                normal_map_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                normal_map_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
            }
        }

        // get emissive texture
        let mut emissive_image = Image::default();
        match material.emissive_texture() {
            None => {
                let bytes = include_bytes!("white.png");
                emissive_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                emissive_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
            }
        }

        // get occlusion texture
        let mut occlusion_image = Image::default();
        match material.occlusion_texture() {
            None => {
                let bytes = include_bytes!("white.png");
                occlusion_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                occlusion_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
            }
        }

        // define the material factors uniform
        let em_factor = material.emissive_factor();
        let em_strength = material.emissive_strength().unwrap_or(1.0);
        let material_factors_uniform = MaterialFactors::new(
            pbr_properties.base_color_factor(),
            [em_factor[0], em_factor[1], em_factor[2], em_strength],
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

        // let pbr_material_factors_bind_group =
        //     device.create_bind_group(&wgpu::BindGroupDescriptor {
        //         layout: pbr_material_factors_bind_group_layout,
        //         entries: &[wgpu::BindGroupEntry {
        //             binding: 10,
        //             resource: pbr_mat_buffer.as_entire_binding(),
        //         }],
        //         label: None,
        //     });

        // define this struct
        Self {
            pbr_material_textures_bind_group: None,
            factor_base_color: material_factors_uniform.base_color.into(),
            factor_emissive: material_factors_uniform.emissive.into(),
            factor_metallic: material_factors_uniform.metallic,
            factor_roughness: material_factors_uniform.roughness,
            factor_alpha: material_factors_uniform.alpha,
            factor_alpha_cutoff: material_factors_uniform.alpha_cutoff,
            alpha_blend_mode: AlphaBlendMode::from(material.alpha_mode()),
            double_sided: material.double_sided(),
            base_color_image,
            metallic_roughness_image,
            normal_map_image,
            emissive_image,
            occlusion_image,
            pbr_mat_buffer,
        }
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pbr_material_textures_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        if self.pbr_material_textures_bind_group.is_some() {
            return;
        }

        if !self.base_color_image.loaded() {
            return;
        }

        if !self.metallic_roughness_image.loaded() {
            return;
        }

        if !self.normal_map_image.loaded() {
            return;
        }

        if !self.emissive_image.loaded() {
            return;
        }

        if !self.occlusion_image.loaded() {
            return;
        }

        // load base color image
        let rgba_image = self.base_color_image.to_rgba8();
        let base_color_texture = crate::texture::Texture::new_with_address_mode(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Base color texture"),
            Some(wgpu::FilterMode::Linear),
            // Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
        )
        .expect("Unable to load base color texture");

        // load metallic image
        let rgba_image = self.metallic_roughness_image.to_rgba8();
        let metallic_roughness_texture = crate::texture::Texture::new_with_address_mode(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Metallic roughness texture"),
            Some(wgpu::FilterMode::Linear),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
        )
        .expect("Unable to load metallic roughness texture");

        // load normal map image
        let rgba_image = self.normal_map_image.to_rgba8();
        let normal_map_texture = crate::texture::Texture::new_with_address_mode(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Normal map texture"),
            Some(wgpu::FilterMode::Linear),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
        )
        .expect("Unable to load normal map texture");

        // load emissive image
        let rgba_image = self.emissive_image.to_rgba8();
        let emissive_texture = crate::texture::Texture::new_with_address_mode(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Emissive texture"),
            Some(wgpu::FilterMode::Linear),
            // Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
        )
        .expect("Unable to load emissive texture");

        // load occlusion image
        let rgba_image = self.occlusion_image.to_rgba8();
        let occlusion_texture = crate::texture::Texture::new_with_address_mode(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Occlusion texture"),
            Some(wgpu::FilterMode::Linear),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
        )
        .expect("Unable to load occlusion texture");

        self.pbr_material_textures_bind_group =
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: pbr_material_textures_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&base_color_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&base_color_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &metallic_roughness_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(
                            &metallic_roughness_texture.sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&normal_map_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&normal_map_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&emissive_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&emissive_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(&occlusion_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::Sampler(&occlusion_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: self.pbr_mat_buffer.as_entire_binding(),
                    },
                ],
                label: Some("pbr_textures_bind_group"),
            }));
    }

    pub fn update_images(&mut self) {
        if !self.base_color_image.loaded() {
            self.base_color_image.update();
        }

        if !self.metallic_roughness_image.loaded() {
            self.metallic_roughness_image.update();
        }

        if !self.normal_map_image.loaded() {
            self.normal_map_image.update();
        }

        if !self.emissive_image.loaded() {
            self.emissive_image.update();
        }

        if !self.occlusion_image.loaded() {
            self.occlusion_image.update();
        }
    }

    pub fn get_progress(&self) -> f32 {
        let base_color_image = self.base_color_image.loaded() as i32;
        let metallic_roughness_image = self.metallic_roughness_image.loaded() as i32;
        let normal_map_image = self.normal_map_image.loaded() as i32;
        let emissive_image = self.emissive_image.loaded() as i32;
        let occlusion_image = self.occlusion_image.loaded() as i32;

        (base_color_image
            + metallic_roughness_image
            + normal_map_image
            + emissive_image
            + occlusion_image) as f32
            / 5.0
    }

    pub fn loaded(&self) -> bool {
        self.pbr_material_textures_bind_group.is_some()
    }
}
