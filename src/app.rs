/**********************************************************************************
 *  Dream is a software for developing real-time 3D experiences.
 *  Copyright (C) 2023 Deepak Ramalingam
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published
 *  by the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 **********************************************************************************/
use gltf::buffer::Source;

use dream_ecs;
use dream_ecs::component::Transform;
use dream_ecs::component_system::ComponentSystem;
use dream_ecs::scene::Scene;

use crate::javascript_script_component_system::JavaScriptScriptComponentSystem;

pub struct App {
    dt: f32,
    scene: Scene,
    javascript_component_system: JavaScriptScriptComponentSystem,
}

impl App {
    pub async fn new() -> Self {
        let dt: f32 = 0.0;
        let mut scene = Scene::new();

        let e = scene.create_entity();
        e.add_transform(Transform::from(dream_math::Vector3::from(1., 1., 1.)));

        let javascript_component_system = JavaScriptScriptComponentSystem::new();

        let path = "cube.glb";
        let gltf = gltf::Gltf::from_slice(
            &dream_resource::load_binary(path)
                .await
                .expect("Error loading binary for glb"),
        )
        .expect("Error loading from slice for glb");
        let mut buffer_data = Vec::new();
        for buffer in gltf.buffers() {
            match buffer.source() {
                Source::Bin => {
                    if let Some(blob) = gltf.blob.as_deref() {
                        buffer_data.push(Vec::from(blob.clone()));
                    };
                }
                Source::Uri(uri) => {
                    let bin = dream_resource::load_binary(uri)
                        .await
                        .expect("unable to load binary");
                    buffer_data.push(bin);
                }
            }
        }

        // let mut meshes = Vec::new();
        for scene in gltf.scenes() {
            println!("scene: {}", scene.name().expect("No name for scene"));
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
                        println!("Mesh for node {}", node.name().expect("No name for node"));
                        println!("{} children for mesh node", node.children().count());
                        let primitives = mesh.primitives();
                        primitives.for_each(|primitive| {
                            let reader =
                                primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
                            let mut vertices = Vec::new();
                            if let Some(vertex_attribute) = reader.read_positions() {
                                vertex_attribute.for_each(|vertex| {
                                    dbg!(vertex);
                                    vertices.push(dream_renderer::model::ModelVertex {
                                        position: vertex,
                                        tex_coords: Default::default(),
                                        normal: Default::default(),
                                    })
                                });
                            }
                            if let Some(normal_attribute) = reader.read_normals() {
                                let mut normal_index = 0;
                                normal_attribute.for_each(|normal| {
                                    dbg!(normal);
                                    vertices[normal_index].normal = normal;

                                    normal_index += 1;
                                });
                            }
                            if let Some(tex_coord_attribute) =
                                reader.read_tex_coords(0).map(|v| v.into_f32())
                            {
                                let mut tex_coord_index = 0;
                                tex_coord_attribute.for_each(|tex_coord| {
                                    dbg!(tex_coord);
                                    vertices[tex_coord_index].tex_coords = tex_coord;

                                    tex_coord_index += 1;
                                });
                            }

                            let mut indices = Vec::new();
                            if let Some(indices_raw) = reader.read_indices() {
                                // dbg!(indices_raw);
                                indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                            }
                            dbg!(indices);

                            // let mesh_name = mesh.name().expect("No mesh name found");
                            // let vertex_buffer =
                            //     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            //         label: Some(&format!("{} {:?} Vertex Buffer", path, mesh_name)),
                            //         contents: bytemuck::cast_slice(&vertices),
                            //         usage: wgpu::BufferUsages::VERTEX,
                            //     });
                            // let index_buffer =
                            //     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            //         label: Some(&format!("{} {:?} Index Buffer", path, mesh_name)),
                            //         contents: bytemuck::cast_slice(&indices),
                            //         usage: wgpu::BufferUsages::INDEX,
                            //     });
                            //
                            // meshes.push(dream_renderer::model::Mesh {
                            //     name: mesh_name.to_string(),
                            //     vertex_buffer,
                            //     index_buffer,
                            //     num_elements: indices.len() as u32,
                            //     // material: m.mesh.material_id.unwrap_or(0),
                            //     material: 0,
                            // });
                        });
                    }
                }
            }
        }

        Self {
            dt,
            scene,
            javascript_component_system,
        }
    }

    pub fn update(&mut self) -> f32 {
        self.dt = 1.0 / 60.0;
        self.javascript_component_system
            .update(self.dt, &mut self.scene);
        return 0.0;
    }
}
