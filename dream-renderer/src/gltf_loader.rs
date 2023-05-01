use gltf::buffer::Source;
use wgpu::util::DeviceExt;

use dream_fs::load_binary;

use crate::material::Material;
use crate::model::{Mesh, Model};

pub async fn read_gltf(
    path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pbr_material_factors_bind_group_layout: &wgpu::BindGroupLayout,
    base_color_texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> Model {
    let gltf = gltf::Gltf::from_slice(
        &load_binary(path)
            .await
            .expect("Error loading binary for glb"),
    )
    .expect("Error loading from slice for glb");
    // let gltf = gltf::Gltf::open("res/cube.gltf").expect("Unable to open gltf");
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            Source::Bin => {
                if let Some(blob) = gltf.blob.as_deref() {
                    buffer_data.push(Vec::from(blob));
                };
            }
            Source::Uri(uri) => {
                let bin = load_binary(uri).await.expect("unable to load binary");
                buffer_data.push(bin);
            }
        }
    }

    // let mut mesh_info = Vec::new();
    let mut meshes = Vec::new();
    let mut materials = Vec::new();

    // get materials for model
    for material in gltf.materials() {
        materials.push(Material::new(
            material,
            device,
            queue,
            pbr_material_factors_bind_group_layout,
            base_color_texture_bind_group_layout,
            &buffer_data,
        ));
    }

    // get meshes for model
    let mut get_dream_mesh = |mesh: gltf::Mesh| {
        mesh.index();
        // println!("Mesh for node {}", node.name().expect("No name for node"));
        // println!("{} children for mesh node", node.children().count());
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
                    })
                });
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

            // mesh_info.push(MeshInfo::new(vertices, indices));

            let mesh_name = mesh.name().expect("No mesh name found");
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} {:?} Vertex Buffer", path, mesh_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} {:?} Index Buffer", path, mesh_name)),
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
    };

    for scene in gltf.scenes() {
        // println!("scene: {}", scene.name().expect("No name for scene"));
        for node in scene.nodes() {
            match node.mesh() {
                None => {
                    for child in node.children() {
                        // TODO: process each child (call method recursively)
                        match child.mesh() {
                            None => {
                                println!("TODO: implement recursive searching method for meshes");
                                for child in child.children() {
                                    match child.mesh() {
                                        None => {
                                            for child in child.children() {
                                                for child in child.children() {
                                                    match child.mesh() {
                                                        None => {
                                                            for child in child.children() {
                                                                for child in child.children() {
                                                                    match child.mesh() {
                                                                        None => {
                                                                            for child in
                                                                                child.children()
                                                                            {
                                                                                for child in
                                                                                    child.children()
                                                                                {
                                                                                    match child.mesh() {
                                                                                        None => {
                                                                                            for child in child.children() {
                                                                                                for child in child.children() {
                                                                                                    match child.mesh() {
                                                                                                        None => {
                                                                                                            for child in child.children() {}
                                                                                                        },
                                                                                                        Some(mesh) => {
                                                                                                            get_dream_mesh(mesh);
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        },
                                                                                        Some(mesh) => {
                                                                                            get_dream_mesh(mesh);
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                        Some(mesh) => {
                                                                            get_dream_mesh(mesh);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Some(mesh) => {
                                                            get_dream_mesh(mesh);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Some(mesh) => {
                                            get_dream_mesh(mesh);
                                        }
                                    }
                                }
                            }
                            Some(mesh) => {
                                get_dream_mesh(mesh);
                            }
                        }
                    }
                }
                Some(mesh) => {
                    get_dream_mesh(mesh);
                }
            }
        }
    }

    Model::new(meshes, materials)
}
