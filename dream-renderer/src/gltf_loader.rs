#[cfg(target_arch = "wasm32")]
use std::io::{Read, Seek, SeekFrom};

use cfg_if::cfg_if;
use gltf::buffer::Source;
use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_file_reader::WebSysFile;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

use crate::model::{Mesh, ModelVertex};
use crate::{model, texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        location.origin().unwrap(),
        option_env!("RES_PATH").unwrap_or("res"),
    ))
    .unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
            // let data = reqwest::get(file_name)
            //     .await?
            //     .bytes()
            //     .await?
            //     .to_vec();
            // TODO: maybe utilize formatting to set a url variable
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

pub async fn read_gltf(path: &str, device: &wgpu::Device) -> Vec<Mesh> {
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
                    buffer_data.push(Vec::from(blob.clone()));
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
    for scene in gltf.scenes() {
        // println!("scene: {}", scene.name().expect("No name for scene"));
        for node in scene.nodes() {
            match node.mesh() {
                None => {
                    for _child in node.children() {
                        // process each child (call method recursively)
                        todo!()
                    }
                }
                Some(mesh) => {
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
                        if let Some(tex_coord_attribute) =
                            reader.read_tex_coords(0).map(|v| v.into_f32())
                        {
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
                        let vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&format!("{} {:?} Vertex Buffer", path, mesh_name)),
                                contents: bytemuck::cast_slice(&vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        let index_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&format!("{} {:?} Index Buffer", path, mesh_name)),
                                contents: bytemuck::cast_slice(&indices),
                                usage: wgpu::BufferUsages::INDEX,
                            });

                        meshes.push(crate::model::Mesh {
                            name: mesh_name.to_string(),
                            vertex_buffer,
                            index_buffer,
                            num_elements: indices.len() as u32,
                            // material: m.mesh.material_id.unwrap_or(0),
                            material: 0,
                        });
                    });
                }
            }
        }
    }

    return meshes;
}
