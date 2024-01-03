use std::ops::Range;

use crate::material::Material;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveInfo {
    pub(crate) num_vertices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],     // 0 , 1,  2
    pub tex_coords: [f32; 2],   // 3,  4
    pub normal: [f32; 3],       // 5,  6,  7
    pub tangent: [f32; 4],      // 8,  9,  10, 11
    pub bone_ids: [u32; 4],     // 12, 13, 14, 15
    pub bone_weights: [f32; 4], // 16, 17, 18, 19
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Primitive {
    pub primitive_info_buffer: wgpu::Buffer,
    pub primitive_info_bind_group: wgpu::BindGroup,
    pub vertex_buffer_bind_group: wgpu::BindGroup,
    pub skinned_vertex_buffer: Option<wgpu::Buffer>,
    pub skinned_vertices_buffer_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
    pub buffer_length: u32,
}

pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Box<Material>>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>, materials: Vec<Box<Material>>) -> Self {
        Self { meshes, materials }
    }
}

pub trait DrawModel<'a> {
    fn draw_primitive_instanced(&mut self, primitive: &'a Primitive, instances: Range<u32>);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_primitive_instanced(&mut self, primitive: &'b Primitive, instances: Range<u32>) {
        if primitive.skinned_vertex_buffer.is_some() {
            self.set_vertex_buffer(
                0,
                primitive.skinned_vertex_buffer.as_ref().unwrap().slice(..),
            );
        } else {
            self.set_vertex_buffer(0, primitive.vertex_buffer.slice(..));
        }
        self.set_index_buffer(primitive.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..primitive.num_elements, 0, instances);
    }
}
