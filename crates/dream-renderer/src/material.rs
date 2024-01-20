use dream_math::{Matrix3, Matrix4, Rotation2, UnitQuaternion, Vector2, Vector3};
use gltf::material::{NormalTexture, OcclusionTexture};
use gltf::texture::{Info, MagFilter, MinFilter, TextureTransform, WrappingMode};
use gltf::Texture;
use std::path::Path;
use wgpu::util::DeviceExt;

use crate::image::Image;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialFactors {
    pub base_color: [f32; 3],
    pub alpha: f32,
    pub emissive: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_cutoff: f32,
    // texture coordinates
    pub base_color_tex_coord: u32,
    pub metallic_roughness_tex_coord: u32,
    pub normal_tex_coord: u32,
    pub emissive_tex_coord: u32,
    pub occlusion_tex_coord: u32,
    // texture transform base color
    pub base_color_tex_transform_0: [f32; 4],
    pub base_color_tex_transform_1: [f32; 4],
    pub base_color_tex_transform_2: [f32; 4],
    // texture transform base color
    pub metallic_roughness_tex_transform_0: [f32; 4],
    pub metallic_roughness_tex_transform_1: [f32; 4],
    pub metallic_roughness_tex_transform_2: [f32; 4],
    // texture transform normal
    pub normal_tex_transform_0: [f32; 4],
    pub normal_tex_transform_1: [f32; 4],
    pub normal_tex_transform_2: [f32; 4],
    // texture transform emissive
    pub emissive_tex_transform_0: [f32; 4],
    pub emissive_tex_transform_1: [f32; 4],
    pub emissive_tex_transform_2: [f32; 4],
    // texture transform occlusion
    pub occlusion_tex_transform_0: [f32; 4],
    pub occlusion_tex_transform_1: [f32; 4],
    pub occlusion_tex_transform_2: [f32; 4],
}

impl MaterialFactors {
    pub fn new(
        base_color: [f32; 4],
        emissive: [f32; 4],
        metallic: f32,
        roughness: f32,
        alpha_cutoff: f32,
        base_color_tex_transform: [[f32; 3]; 3],
        metallic_roughness_tex_transform: [[f32; 3]; 3],
        normal_tex_transform: [[f32; 3]; 3],
        emissive_tex_transform: [[f32; 3]; 3],
        occlusion_tex_transform: [[f32; 3]; 3],
        base_color_tex_coord: u32,
        metallic_roughness_tex_coord: u32,
        normal_tex_coord: u32,
        emissive_tex_coord: u32,
        occlusion_tex_coord: u32,
    ) -> Self {
        Self {
            base_color: [
                *base_color.get(0).unwrap(),
                *base_color.get(1).unwrap(),
                *base_color.get(2).unwrap(),
            ],
            alpha: *(base_color.get(3).unwrap_or(&1.0)),
            emissive,
            metallic,
            roughness,
            alpha_cutoff,
            base_color_tex_coord,
            metallic_roughness_tex_coord,
            normal_tex_coord,
            emissive_tex_coord,
            occlusion_tex_coord,
            base_color_tex_transform_0: [
                base_color_tex_transform[0][0],
                base_color_tex_transform[0][1],
                base_color_tex_transform[0][2],
                1.0,
            ],
            base_color_tex_transform_1: [
                base_color_tex_transform[1][0],
                base_color_tex_transform[1][1],
                base_color_tex_transform[1][2],
                1.0,
            ],
            base_color_tex_transform_2: [
                base_color_tex_transform[2][0],
                base_color_tex_transform[2][1],
                base_color_tex_transform[2][2],
                1.0,
            ],
            metallic_roughness_tex_transform_0: [
                metallic_roughness_tex_transform[0][0],
                metallic_roughness_tex_transform[0][1],
                metallic_roughness_tex_transform[0][2],
                1.0,
            ],
            metallic_roughness_tex_transform_1: [
                metallic_roughness_tex_transform[1][0],
                metallic_roughness_tex_transform[1][1],
                metallic_roughness_tex_transform[1][2],
                1.0,
            ],
            metallic_roughness_tex_transform_2: [
                metallic_roughness_tex_transform[2][0],
                metallic_roughness_tex_transform[2][1],
                metallic_roughness_tex_transform[2][2],
                1.0,
            ],
            normal_tex_transform_0: [
                normal_tex_transform[0][0],
                normal_tex_transform[0][1],
                normal_tex_transform[0][2],
                1.0,
            ],
            normal_tex_transform_1: [
                normal_tex_transform[1][0],
                normal_tex_transform[1][1],
                normal_tex_transform[1][2],
                1.0,
            ],
            normal_tex_transform_2: [
                normal_tex_transform[2][0],
                normal_tex_transform[2][1],
                normal_tex_transform[2][2],
                1.0,
            ],
            emissive_tex_transform_0: [
                emissive_tex_transform[0][0],
                emissive_tex_transform[0][1],
                emissive_tex_transform[0][2],
                1.0,
            ],
            emissive_tex_transform_1: [
                emissive_tex_transform[1][0],
                emissive_tex_transform[1][1],
                emissive_tex_transform[1][2],
                1.0,
            ],
            emissive_tex_transform_2: [
                emissive_tex_transform[2][0],
                emissive_tex_transform[2][1],
                emissive_tex_transform[2][2],
                1.0,
            ],
            occlusion_tex_transform_0: [
                occlusion_tex_transform[0][0],
                occlusion_tex_transform[0][1],
                occlusion_tex_transform[0][2],
                1.0,
            ],
            occlusion_tex_transform_1: [
                occlusion_tex_transform[1][0],
                occlusion_tex_transform[1][1],
                occlusion_tex_transform[1][2],
                1.0,
            ],
            occlusion_tex_transform_2: [
                occlusion_tex_transform[2][0],
                occlusion_tex_transform[2][1],
                occlusion_tex_transform[2][2],
                1.0,
            ],
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

pub struct TextureInfo {
    pub address_mode_u: wgpu::AddressMode,
    pub address_mode_v: wgpu::AddressMode,
    pub address_mode_w: wgpu::AddressMode,
    pub mag_filter: wgpu::FilterMode,
    pub min_filter: wgpu::FilterMode,
}

impl Default for TextureInfo {
    fn default() -> Self {
        Self {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
        }
    }
}

fn address_mode_for_gltf_wrapping_mode(
    wrapping_mode: gltf::texture::WrappingMode,
) -> wgpu::AddressMode {
    return match wrapping_mode {
        WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        WrappingMode::Repeat => wgpu::AddressMode::Repeat,
    };
}

fn filter_for_gltf_min_filter(filter: gltf::texture::MinFilter) -> wgpu::FilterMode {
    return match filter {
        MinFilter::Nearest => wgpu::FilterMode::Nearest,
        MinFilter::Linear => wgpu::FilterMode::Linear,
        MinFilter::NearestMipmapNearest => wgpu::FilterMode::Nearest,
        MinFilter::LinearMipmapNearest => wgpu::FilterMode::Linear,
        MinFilter::NearestMipmapLinear => wgpu::FilterMode::Nearest,
        MinFilter::LinearMipmapLinear => wgpu::FilterMode::Linear,
    };
}

fn filter_for_gltf_mag_filter(filter: gltf::texture::MagFilter) -> wgpu::FilterMode {
    return match filter {
        MagFilter::Nearest => wgpu::FilterMode::Nearest,
        MagFilter::Linear => wgpu::FilterMode::Linear,
    };
}

impl<'a> From<Texture<'a>> for TextureInfo {
    fn from(gltf_texture: Texture) -> Self {
        let address_mode_u = gltf_texture.sampler().wrap_s();
        let address_mode_v = gltf_texture.sampler().wrap_t();
        let address_mode_w = gltf::texture::WrappingMode::Repeat;
        let mag_filter = gltf_texture
            .sampler()
            .mag_filter()
            .unwrap_or(gltf::texture::MagFilter::Linear);
        let min_filter = gltf_texture
            .sampler()
            .min_filter()
            .unwrap_or(gltf::texture::MinFilter::Nearest);
        Self {
            address_mode_u: address_mode_for_gltf_wrapping_mode(address_mode_u),
            address_mode_v: address_mode_for_gltf_wrapping_mode(address_mode_v),
            address_mode_w: address_mode_for_gltf_wrapping_mode(address_mode_w),
            mag_filter: filter_for_gltf_mag_filter(mag_filter),
            min_filter: filter_for_gltf_min_filter(min_filter),
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
    pub base_color_texture_info: TextureInfo,
    pub metallic_roughness_texture_info: TextureInfo,
    pub normal_texture_info: TextureInfo,
    pub emissive_texture_info: TextureInfo,
    pub occlusion_texture_info: TextureInfo,
}

fn mat3_from_texture_transform(texture_transform: &TextureTransform) -> Matrix3<f32> {
    Matrix3::<f32>::new_translation(&texture_transform.offset().into())
        * Rotation2::new(-1.0 * texture_transform.rotation()).to_homogeneous()
        * Matrix3::new_nonuniform_scaling(&texture_transform.scale().into())
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
        let mut base_color_transform: Matrix3<f32> = Matrix3::<f32>::identity();
        let mut base_color_texture_info: TextureInfo = TextureInfo::default();
        let mut base_color_tex_coord = 0;
        match pbr_properties.base_color_texture() {
            None => {
                let bytes = include_bytes!("white.png");
                base_color_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                if let Some(texture_transform) = texture_info.texture_transform() {
                    base_color_transform = mat3_from_texture_transform(&texture_transform);
                }
                base_color_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
                base_color_texture_info = texture_info.texture().into();
                base_color_tex_coord = texture_info.tex_coord();
            }
        }

        // get metallic texture
        let mut metallic_roughness_image = Image::default();
        let mut metallic_roughness_transform: Matrix3<f32> = Matrix3::<f32>::identity();
        let mut metallic_roughness_texture_info: TextureInfo = TextureInfo::default();
        let mut metallic_roughness_tex_coord = 0;
        match pbr_properties.metallic_roughness_texture() {
            None => {
                let bytes = include_bytes!("black.png");
                metallic_roughness_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                if let Some(texture_transform) = texture_info.texture_transform() {
                    metallic_roughness_transform = mat3_from_texture_transform(&texture_transform);
                }
                metallic_roughness_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
                metallic_roughness_texture_info = texture_info.texture().into();
                metallic_roughness_tex_coord = texture_info.tex_coord();
            }
        }

        // get normal map texture
        let mut normal_map_image = Image::default();
        let mut normal_texture_info = TextureInfo::default();
        let mut normal_tex_coord = 0;
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
                normal_texture_info = texture_info.texture().into();
                normal_tex_coord = texture_info.tex_coord();
            }
        }

        // get emissive texture
        let mut emissive_image = Image::default();
        let mut emissive_transform: Matrix3<f32> = Matrix3::<f32>::identity();
        let mut emissive_texture_info = TextureInfo::default();
        let mut emissive_tex_coord = 0;
        match material.emissive_texture() {
            None => {
                let bytes = include_bytes!("white.png");
                emissive_image.load_from_bytes_threaded(bytes, "default", None);
            }
            Some(texture_info) => {
                if let Some(texture_transform) = texture_info.texture_transform() {
                    emissive_transform = mat3_from_texture_transform(&texture_transform);
                }
                emissive_image.load_from_gltf_texture_threaded(
                    texture_info.texture(),
                    buffer_data,
                    image_folder,
                );
                emissive_texture_info = texture_info.texture().into();
                emissive_tex_coord = texture_info.tex_coord();
            }
        }

        // get occlusion texture
        let mut occlusion_image = Image::default();
        let mut occlusion_texture_info = TextureInfo::default();
        let mut occlusion_tex_coord = 0;
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
                occlusion_texture_info = texture_info.texture().into();
                occlusion_tex_coord = texture_info.tex_coord();
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
            material.alpha_cutoff().unwrap_or(0.5),
            base_color_transform.into(),
            metallic_roughness_transform.into(),
            base_color_transform.into(),
            emissive_transform.into(),
            base_color_transform.into(),
            base_color_tex_coord,
            metallic_roughness_tex_coord,
            normal_tex_coord,
            emissive_tex_coord,
            occlusion_tex_coord,
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
            base_color_texture_info,
            metallic_roughness_texture_info,
            normal_texture_info,
            emissive_texture_info,
            occlusion_texture_info,
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
        let base_color_texture = crate::texture::Texture::new_with_address_mode_and_filters(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Base color texture"),
            Some(wgpu::FilterMode::Linear),
            // Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            self.base_color_texture_info.address_mode_u,
            self.base_color_texture_info.address_mode_v,
            self.base_color_texture_info.address_mode_w,
            self.base_color_texture_info.min_filter,
            self.base_color_texture_info.mag_filter,
        )
        .expect("Unable to load base color texture");

        // load metallic image
        let rgba_image = self.metallic_roughness_image.to_rgba8();
        let metallic_roughness_texture =
            crate::texture::Texture::new_with_address_mode_and_filters(
                device,
                queue,
                rgba_image.to_vec(),
                rgba_image.dimensions(),
                Some("Metallic roughness texture"),
                Some(wgpu::FilterMode::Linear),
                Some(wgpu::TextureFormat::Rgba8Unorm),
                self.metallic_roughness_texture_info.address_mode_u,
                self.metallic_roughness_texture_info.address_mode_v,
                self.metallic_roughness_texture_info.address_mode_w,
                self.metallic_roughness_texture_info.min_filter,
                self.metallic_roughness_texture_info.mag_filter,
            )
            .expect("Unable to load metallic roughness texture");

        // load normal map image
        let rgba_image = self.normal_map_image.to_rgba8();
        let normal_map_texture = crate::texture::Texture::new_with_address_mode_and_filters(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Normal map texture"),
            Some(wgpu::FilterMode::Linear),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            self.normal_texture_info.address_mode_u,
            self.normal_texture_info.address_mode_v,
            self.normal_texture_info.address_mode_w,
            self.normal_texture_info.min_filter,
            self.normal_texture_info.mag_filter,
        )
        .expect("Unable to load normal map texture");

        // load emissive image
        let rgba_image = self.emissive_image.to_rgba8();
        let emissive_texture = crate::texture::Texture::new_with_address_mode_and_filters(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Emissive texture"),
            Some(wgpu::FilterMode::Linear),
            // Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            self.emissive_texture_info.address_mode_u,
            self.emissive_texture_info.address_mode_v,
            self.emissive_texture_info.address_mode_w,
            self.emissive_texture_info.min_filter,
            self.emissive_texture_info.mag_filter,
        )
        .expect("Unable to load emissive texture");

        // load occlusion image
        let rgba_image = self.occlusion_image.to_rgba8();
        let occlusion_texture = crate::texture::Texture::new_with_address_mode_and_filters(
            device,
            queue,
            rgba_image.to_vec(),
            rgba_image.dimensions(),
            Some("Occlusion texture"),
            Some(wgpu::FilterMode::Linear),
            Some(wgpu::TextureFormat::Rgba8Unorm),
            self.occlusion_texture_info.address_mode_u,
            self.occlusion_texture_info.address_mode_v,
            self.occlusion_texture_info.address_mode_w,
            self.occlusion_texture_info.min_filter,
            self.occlusion_texture_info.mag_filter,
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
