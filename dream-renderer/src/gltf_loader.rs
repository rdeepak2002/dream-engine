use gltf::buffer::Source;
use wgpu::util::DeviceExt;

use dream_fs::load_binary;

use crate::model::{MaterialUniform, Mesh, Model};

pub async fn read_gltf(
    path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
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
        let pbr_properties = material.pbr_metallic_roughness();
        if material
            .pbr_metallic_roughness()
            .base_color_texture()
            .is_some()
        {
            let tex = pbr_properties
                .base_color_texture()
                .expect("No base color texture")
                .texture();
            let tex_name = tex.name().unwrap_or("No texture name");
            // println!("texture name {}", tex_name);
            let texture_source = tex.source().source();
            match texture_source {
                gltf::image::Source::View { view, mime_type } => {
                    // TODO: below is wrong for sure (since the buffer size is always the same)
                    // let buf_dat = &buffer_data[view.buffer().index()];
                    let parent_buffer_data = &buffer_data[view.buffer().index()];
                    let begin = view.offset();
                    let end = view.offset() + view.length();
                    let buf_dat = &parent_buffer_data[begin..end];
                    // println!("mime_type is {}", mime_type);
                    // println!("buffer length {}", buf_dat.len());
                    // load texture from binary
                    let mime_type = Some(mime_type.to_string());
                    let base_color_texture = crate::texture::Texture::from_bytes(
                        device, queue, buf_dat, tex_name, mime_type,
                    )
                    .expect("Couldn't load base color texture");
                }
                gltf::image::Source::Uri { uri, mime_type } => {
                    todo!();
                    // let base_color_texture = crate::texture::Texture::load_texture(uri, device, queue).await;
                }
            };
        }
        // get base_color for PBR
        let base_color = pbr_properties.base_color_factor();
        let red = *base_color.first().expect("No red found for base color");
        let green = *base_color.get(1).expect("No green found for base color");
        let blue = *base_color.get(2).expect("No blue found for base color");
        let alpha = *base_color.get(3).expect("No alpha found for base color");
        // let base_color = cgmath::Vector4::new(1.0, 1.0, 0.0, 1.0).into();        // <- TODO: this works, but not the bottom line of code...
        let base_color = cgmath::Vector4::new(red, green, blue, alpha).into();
        // let base_color = cgmath::Vector4::new(red, green, blue, 1.0).into();
        // println!(
        //     "base_color: (r {}, g {}, b {}, a {})",
        //     red, green, blue, alpha
        // );
        println!(
            "TODO: sample base color texture too (refer to old code on how to sample texture)"
        );
        // TODO: maybe we need to sample the base color texture?
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
        materials.push(crate::model::Material {
            base_color,
            bind_group,
        });
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
