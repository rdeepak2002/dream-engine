use std::collections::HashMap;

use gltf::buffer::Source;
use gltf::Mesh;
use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use dream_fs::fs::read_binary;

use crate::material::Material;
use crate::model::{Model, ModelVertex};

pub fn read_gltf<'a>(
    path: &str,
    device: &wgpu::Device,
    pbr_material_factors_bind_group_layout: &wgpu::BindGroupLayout,
) -> Model {
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
                let bin = read_binary(std::path::PathBuf::from(uri), false)
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
        )));
    }

    let mut mesh_list = Vec::new();
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            process_gltf_child_node(node, &mut mesh_list);
        }
    }

    // use mesh map to keep track of which indices correspond to meshes to have consistency
    // between mesh loading in scene view and mesh indices of loaded model
    let mut mesh_map = HashMap::new();
    for mesh in mesh_list {
        let idx = mesh.index();
        for mesh in get_dream_meshes_from_gltf_mesh(device, mesh, &buffer_data) {
            mesh_map.insert(idx, mesh);
        }
    }
    let mut meshes = Vec::new();
    for i in 0..mesh_map.len() {
        meshes.push(mesh_map.remove(&i).unwrap());
    }

    Model::new(meshes, materials)
}

fn process_gltf_child_node<'a>(child_node: gltf::Node<'a>, mesh_list: &mut Vec<Mesh<'a>>) {
    match child_node.mesh() {
        None => {
            for child in child_node.children() {
                process_gltf_child_node(child, mesh_list);
            }
        }
        Some(mesh) => {
            mesh_list.push(mesh);
        }
    }
}

// logic for using MikkTSpace algorithm for computing tangents
// not being used due to how long it takes to run
pub type Face = [u32; 3];

struct MkktSpaceMesh {
    faces: Vec<Face>,
    vertices: Vec<ModelVertex>,
}

fn vertex(mesh: &MkktSpaceMesh, face: usize, vert: usize) -> &ModelVertex {
    let vs: &[u32; 3] = &mesh.faces[face];
    &mesh.vertices[vs[vert] as usize]
}

impl mikktspace::Geometry for MkktSpaceMesh {
    fn num_faces(&self) -> usize {
        self.faces.len()
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        vertex(self, face, vert).position.into()
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        vertex(self, face, vert).normal.into()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        vertex(self, face, vert).tex_coords.into()
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let vs: Face = self.faces[face];
        self.vertices[vs[vert] as usize].tangent = tangent;
    }
}

fn get_dream_meshes_from_gltf_mesh(
    device: &wgpu::Device,
    mesh: Mesh,
    buffer_data: &Vec<Vec<u8>>,
) -> Vec<crate::model::Mesh> {
    let mut meshes = Vec::new();
    let primitives = mesh.primitives();
    primitives.for_each(|primitive| {
        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
        let mut vertices = Vec::new();
        if let Some(vertex_attribute) = reader.read_positions() {
            vertex_attribute.for_each(|vertex| {
                // dbg!(vertex);
                vertices.push(crate::model::ModelVertex {
                    position: vertex,
                    tex_coords: Default::default(),
                    normal: Default::default(),
                    tangent: [0.0, 0.0, 0.0, 0.0],
                })
            });
        }
        let mut manually_compute_tangents = false;
        if let Some(tangent_attribute) = reader.read_tangents() {
            let mut tangent_index = 0;
            tangent_attribute.for_each(|tangent| {
                vertices[tangent_index].tangent = tangent;

                tangent_index += 1;
            });
        } else {
            manually_compute_tangents = true;
        }
        if let Some(normal_attribute) = reader.read_normals() {
            let mut normal_index = 0;
            normal_attribute.for_each(|normal| {
                // dbg!(normal);
                vertices[normal_index].normal = normal;

                normal_index += 1;
            });
        }
        if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
            let mut tex_coord_index = 0;
            tex_coord_attribute.for_each(|tex_coord| {
                // dbg!(tex_coord);
                vertices[tex_coord_index].tex_coords = tex_coord;

                tex_coord_index += 1;
            });
        }

        let mut indices = Vec::new();
        if let Some(indices_raw) = reader.read_indices() {
            // dbg!(indices_raw);
            indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
        }

        if manually_compute_tangents {
            // commented-out example of how to use MikkTSpace algorithm
            // let mut faces = Vec::<Face>::new();
            // for i in (0..indices.len()).step_by(3) {
            //     faces.push([indices[i], indices[i + 1], indices[i + 2]]);
            // }
            // let mut m = MkktSpaceMesh {
            //     vertices: vertices.clone(),
            //     faces: faces.clone(),
            // };
            // let ret = mikktspace::generate_tangents(&mut m);
            // assert!(ret);
            // vertices = m.vertices;
            for i in (0..indices.len()).step_by(3) {
                let v0 = vertices[indices[i] as usize];
                let v1 = vertices[indices[i + 1] as usize];
                let v2 = vertices[indices[i + 2] as usize];

                let edge1 = Vector3::from(v1.position) - Vector3::from(v0.position);
                let edge2 = Vector3::from(v2.position) - Vector3::from(v0.position);

                let delta_u1 = v1.tex_coords[0] - v0.tex_coords[0];
                let delta_v1 = v1.tex_coords[1] - v0.tex_coords[1];
                let delta_u2 = v2.tex_coords[0] - v0.tex_coords[0];
                let delta_v2 = v2.tex_coords[1] - v0.tex_coords[1];

                let f = 1.0 / (delta_u1 * delta_v2 - delta_u2 * delta_v1);

                let mut tangent = Vector3::new(0., 0., 0.);

                tangent.x = f * (delta_v2 * edge1.x - delta_v1 * edge2.x);
                tangent.y = f * (delta_v2 * edge1.y - delta_v1 * edge2.y);
                tangent.z = f * (delta_v2 * edge1.z - delta_v1 * edge2.z);

                vertices[indices[i] as usize].tangent = [tangent.x, tangent.y, tangent.z, 1.0];
                vertices[indices[i + 1] as usize].tangent = [tangent.x, tangent.y, tangent.z, 1.0];
                vertices[indices[i + 2] as usize].tangent = [tangent.x, tangent.y, tangent.z, 1.0];
            }
        }

        let mesh_name = mesh.name().expect("No mesh name found");
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{mesh_name} Vertex Buffer")),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{mesh_name} Index Buffer")),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        meshes.push(crate::model::Mesh {
            name: mesh_name.to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: primitive.material().index().unwrap_or(0),
        });
    });
    meshes
}
