use std::collections::HashMap;

use dream_ecs::component::MeshRenderer;
use gltf::buffer::Source;
use gltf::Mesh;
use wgpu::util::DeviceExt;

use dream_fs::fs::read_binary;
use dream_math::Vector3;

use crate::material::Material;
use crate::model::{Model, ModelVertex, PrimitiveInfo};

pub fn read_gltf<'a>(
    path: &str,
    device: &wgpu::Device,
    pbr_material_factors_bind_group_layout: &wgpu::BindGroupLayout,
) -> Model {
    let current_gltf_path = std::path::PathBuf::from(path);
    let parent_folder = current_gltf_path.as_path().parent().unwrap();
    let gltf = gltf::Gltf::from_slice(
        &read_binary(std::path::PathBuf::from(path), true)
            .unwrap_or_else(|_| panic!("Error loading binary for glb {}", path)),
    )
    .expect("Error loading from slice for glb");
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            Source::Bin => {
                if let Some(blob) = gltf.blob.as_deref() {
                    buffer_data.push(Vec::from(blob));
                };
            }
            Source::Uri(uri) => {
                let final_path = parent_folder.join(std::path::PathBuf::from(uri));
                let bin = read_binary(final_path, true)
                    .unwrap_or_else(|_| panic!("unable to load binary at uri {}", uri));
                buffer_data.push(bin);
            }
        }
    }

    // let mut mesh_info = Vec::new();
    let mut materials = Vec::new();

    // get materials for model
    for material in gltf.materials() {
        materials.push(Box::new(Material::new(
            material,
            device,
            pbr_material_factors_bind_group_layout,
            &buffer_data,
            parent_folder,
        )));
    }

    let mut mesh_list = Vec::new();
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            if node.mesh().is_some() {
                mesh_list.push(node.mesh().unwrap());
            }
            process_gltf_child_node(node, &mut mesh_list);
        }
    }

    // use mesh map to keep track of which indices correspond to meshes to have consistency
    // between mesh loading in scene view and mesh indices of loaded model
    let mut mesh_map = HashMap::new();
    for mesh in mesh_list {
        let idx = mesh.index();
        let mesh = crate::model::Mesh {
            name: mesh.name().unwrap_or("mesh").to_string(),
            primitives: get_dream_primitives_from_gltf_mesh(device, mesh, &buffer_data),
        };
        log::debug!("Mesh loaded name: {:?} idx: {:?}", mesh.name, idx);
        mesh_map.insert(idx, mesh);
    }
    let mut meshes = Vec::new();
    for i in 0..mesh_map.len() {
        if mesh_map.contains_key(&i) {
            meshes.push(Some(mesh_map.remove(&i).unwrap()));
        } else {
            // TODO: this line shouldnt even be triggered
            log::error!("Unable to remove mesh with key {i}");
            meshes.push(None);
        }
    }

    Model::new(meshes, materials)
}

fn process_gltf_child_node<'a>(child_node: gltf::Node<'a>, mesh_list: &mut Vec<Mesh<'a>>) {
    for child in child_node.children() {
        if child.mesh().is_some() {
            mesh_list.push(child.mesh().unwrap());
        }
        process_gltf_child_node(child, mesh_list);
    }
}

// logic for using MikkTSpace algorithm for computing tangents
// not being used due to how long it takes to run
pub type Face = [u32; 3];

struct MeshVerticesAndIndicesContainer {
    indices: Vec<u32>,
    vertices: Vec<ModelVertex>,
}

impl MeshVerticesAndIndicesContainer {
    fn vertex(&self, face: usize, vert: usize) -> &ModelVertex {
        &self.vertices[self.indices[face * 3 + vert] as usize]
    }
}

impl mikktspace::Geometry for MeshVerticesAndIndicesContainer {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertex(face, vert).position
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertex(face, vert).normal
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.vertex(face, vert).tex_coords_0
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        self.vertices[self.indices[face * 3 + vert] as usize].tangent = tangent;
    }
}

fn get_dream_primitives_from_gltf_mesh(
    device: &wgpu::Device,
    mesh: Mesh,
    buffer_data: &Vec<Vec<u8>>,
) -> Vec<crate::model::Primitive> {
    let mut primitives_result = Vec::new();
    let primitives = mesh.primitives();
    // log::debug!("Number of primitives is {}", primitives.len());
    primitives.for_each(|primitive| {
        let mut mesh_vertices_and_indices = MeshVerticesAndIndicesContainer {
            vertices: Vec::new(),
            indices: Vec::new(),
        };

        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
        if let Some(vertex_attribute) = reader.read_positions() {
            vertex_attribute.for_each(|vertex| {
                mesh_vertices_and_indices
                    .vertices
                    .push(crate::model::ModelVertex {
                        position: vertex,
                        tex_coords_0: Default::default(),
                        normal: Default::default(),
                        tangent: [0.0, 0.0, 0.0, 0.0],
                        bone_ids: [0, 0, 0, 0],
                        bone_weights: [0., 0., 0., 0.],
                        color: [1.0, 1.0, 1.0, 1.0],
                        tex_coords_1: Default::default(),
                    })
            });
            // log::debug!(
            //     "Number of vertices: {:?}",
            //     mesh_vertices_and_indices.vertices.len()
            // );
        }

        if let Some(color_attribute) = reader.read_colors(0).map(|v| v.into_rgba_f32()) {
            let mut color_index = 0;
            color_attribute.for_each(|color| {
                mesh_vertices_and_indices.vertices[color_index].color = color;

                color_index += 1;
            });
        }

        let mut manually_compute_tangents = false;
        if let Some(tangent_attribute) = reader.read_tangents() {
            let mut tangent_index = 0;
            tangent_attribute.for_each(|tangent| {
                mesh_vertices_and_indices.vertices[tangent_index].tangent = tangent;

                tangent_index += 1;
            });
        } else {
            manually_compute_tangents = true;
        }
        if let Some(normal_attribute) = reader.read_normals() {
            let mut normal_index = 0;
            normal_attribute.for_each(|normal| {
                mesh_vertices_and_indices.vertices[normal_index].normal = normal;

                normal_index += 1;
            });
        }
        if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
            let mut tex_coord_index = 0;
            tex_coord_attribute.for_each(|tex_coord| {
                mesh_vertices_and_indices.vertices[tex_coord_index].tex_coords_0 = tex_coord;

                tex_coord_index += 1;
            });
        }

        if let Some(tex_coord_attribute) = reader.read_tex_coords(1).map(|v| v.into_f32()) {
            let mut tex_coord_index = 0;
            tex_coord_attribute.for_each(|tex_coord| {
                mesh_vertices_and_indices.vertices[tex_coord_index].tex_coords_1 = tex_coord;

                tex_coord_index += 1;
            });
        }

        if let Some(indices_raw) = reader.read_indices() {
            for idx in indices_raw.into_u32() {
                mesh_vertices_and_indices.indices.push(idx);
            }
        }

        // joints and weights for vertex skinning / skeletal animation
        // TODO: I'm passing wrong thing as index / set field of read_
        // let mut is_skinned = false;
        if let Some(joints) = reader.read_joints(0) {
            // is_skinned = true;
            let mut joint_index = 0;
            joints.into_u16().for_each(|joint| {
                mesh_vertices_and_indices.vertices[joint_index].bone_ids = [
                    joint[0] as u32,
                    joint[1] as u32,
                    joint[2] as u32,
                    joint[3] as u32,
                ];
                joint_index += 1;
            });
        }

        // TODO: I'm passing wrong thing as index / set field of read_
        if let Some(weights) = reader.read_weights(0) {
            let mut weight_index = 0;
            weights.into_f32().for_each(|weight| {
                let w1 = weight[0];
                let w2 = weight[1];
                let w3 = weight[2];
                let w4 = weight[3];
                let w_sum = w1 + w2 + w3 + w4;
                if w_sum > 0.0 {
                    mesh_vertices_and_indices.vertices[weight_index].bone_weights =
                        [w1 / w_sum, w2 / w_sum, w3 / w_sum, w4 / w_sum];
                } else {
                    mesh_vertices_and_indices.vertices[weight_index].bone_weights =
                        [w1, w2, w3, w4];
                }
                weight_index += 1;
            });
        }

        let use_mikktspace_algorithm = false;

        if manually_compute_tangents {
            if use_mikktspace_algorithm {
                assert!(mikktspace::generate_tangents(
                    &mut mesh_vertices_and_indices
                ));
            } else {
                for i in (0..mesh_vertices_and_indices.indices.len()).step_by(3) {
                    let v0 = mesh_vertices_and_indices.vertices
                        [mesh_vertices_and_indices.indices[i] as usize];
                    let v1 = mesh_vertices_and_indices.vertices
                        [mesh_vertices_and_indices.indices[i + 1] as usize];
                    let v2 = mesh_vertices_and_indices.vertices
                        [mesh_vertices_and_indices.indices[i + 2] as usize];

                    let edge1 = Vector3::from(v1.position) - Vector3::from(v0.position);
                    let edge2 = Vector3::from(v2.position) - Vector3::from(v0.position);

                    // TODO: when should we consider tex_coords_1? - maybe store another set of tangents...?
                    let delta_u1 = v1.tex_coords_0[0] - v0.tex_coords_0[0];
                    let delta_v1 = v1.tex_coords_0[1] - v0.tex_coords_0[1];
                    let delta_u2 = v2.tex_coords_0[0] - v0.tex_coords_0[0];
                    let delta_v2 = v2.tex_coords_0[1] - v0.tex_coords_0[1];

                    let f = 1.0 / (delta_u1 * delta_v2 - delta_u2 * delta_v1);

                    let mut tangent = Vector3::new(0., 0., 0.);

                    tangent.x = f * (delta_v2 * edge1.x - delta_v1 * edge2.x);
                    tangent.y = f * (delta_v2 * edge1.y - delta_v1 * edge2.y);
                    tangent.z = f * (delta_v2 * edge1.z - delta_v1 * edge2.z);

                    // TODO: This w is incorrect I think?

                    mesh_vertices_and_indices.vertices
                        [mesh_vertices_and_indices.indices[i] as usize]
                        .tangent = [tangent.x, tangent.y, tangent.z, 1.0];
                    mesh_vertices_and_indices.vertices
                        [mesh_vertices_and_indices.indices[i + 1] as usize]
                        .tangent = [tangent.x, tangent.y, tangent.z, 1.0];
                    mesh_vertices_and_indices.vertices
                        [mesh_vertices_and_indices.indices[i + 2] as usize]
                        .tangent = [tangent.x, tangent.y, tangent.z, 1.0];
                }
            }
        }

        let mesh_name = mesh.name().unwrap_or("mesh");

        // TODO: create a new() method for Primitive struct to do all this logic
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{mesh_name} Vertex Buffer")),
            contents: bytemuck::cast_slice(&mesh_vertices_and_indices.vertices),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{mesh_name} Index Buffer")),
            contents: bytemuck::cast_slice(&mesh_vertices_and_indices.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let primitive_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Primitive Info"),
            contents: bytemuck::cast_slice(&[PrimitiveInfo {
                num_vertices: mesh_vertices_and_indices.vertices.len() as u32,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let input: &[u8] = bytemuck::cast_slice(&mesh_vertices_and_indices.vertices);
        let primitive_info_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("primitive_info_bind_group_layout"),
            });
        let primitive_info_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &primitive_info_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: primitive_info_buffer.as_entire_binding(),
            }],
            label: Some("primitive_info_bind_group"),
        });
        let vertices_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                    },
                }],
            });
        let skinned_vertices_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                    },
                }],
            });
        let vertex_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vertices buffer bind group"),
            layout: &vertices_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_buffer.as_entire_binding(),
            }],
        });
        let mut skinned_vertex_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertices Input"),
                contents: bytemuck::cast_slice(input.clone()),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            },
        ));
        let skinned_vertices_buffer_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("skinning vertices buffer bind group"),
                layout: &skinned_vertices_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: skinned_vertex_buffer.as_mut().unwrap().as_entire_binding(),
                }],
            });
        primitives_result.push(crate::model::Primitive {
            vertex_buffer,
            vertex_buffer_bind_group,
            skinned_vertex_buffer,
            skinned_vertices_buffer_bind_group,
            primitive_info_buffer,
            primitive_info_bind_group,
            index_buffer,
            num_elements: mesh_vertices_and_indices.indices.len() as u32,
            material: primitive.material().index().unwrap_or(0),
            buffer_length: input.len() as u32,
        });
    });
    primitives_result
}
