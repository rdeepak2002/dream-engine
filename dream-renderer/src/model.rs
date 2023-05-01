use std::ops::Range;

use wgpu::util::DeviceExt;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

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
    bind_group: wgpu::BindGroup,
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

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>, materials: Vec<Material>) -> Self {
        Self { meshes, materials }
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, material: &'a Material, mesh: &'a Mesh);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, material: &'b Material, mesh: &'b Mesh) {
        self.draw_mesh_instanced(mesh, material, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    // fn draw_model(&mut self, model: &'b Model, camera_bind_group: &'b wgpu::BindGroup) {
    //     self.draw_model_instanced(model, 0..1, camera_bind_group);
    // }
    //
    // fn draw_model_instanced(
    //     &mut self,
    //     model: &'b Model,
    //     instances: Range<u32>,
    //     camera_bind_group: &'b wgpu::BindGroup,
    // ) {
    //     for mesh in &model.meshes {
    //         log::warn!("materials: {}", model.materials.len());
    //         let material = &model.materials[mesh.material];
    //         self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group);
    //     }
    // }
}
