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

        // let path = "res/cube.glb";
        // let file =
        //     std::fs::File::open(&path).expect("Unable to open or find file at specified path");
        // let reader = std::io::BufReader::new(file);
        // let gltf = gltf::Gltf::from_reader(reader).expect("Unable to read Gltf file");
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
                    // TODO: allow synchronous loading
                    let bin = dream_resource::load_binary(uri)
                        .await
                        .expect("unable to load binary");
                    buffer_data.push(bin);
                }
            }
        }
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

                            if let Some(vertex_attibute) = reader.read_positions().map(|v| {
                                dbg!(v);
                            }) {
                                // Save the position here using mapped vertex_attribute result
                            }
                            // let material = primitive.material().index();
                            // let indices = primitive.indices();
                            // let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
                            // if let Some(vertex_attibute) = reader.read_positions().map(|v| {
                            //     dbg!(v);
                            // }) {
                            //     // Save the position here using mapped vertex_attribute result
                            // }
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
