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

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, Weak};

use anyhow::{anyhow, Result};
use gltf::buffer::Source;
use shipyard::{IntoIter, IntoWithId};

use dream_fs::fs::read_binary;
use dream_math::Matrix4;
use dream_resource::resource_manager::ResourceManager;

use crate::component::{Bone, Hierarchy, MeshRenderer, Tag, Transform};
use crate::entity::Entity;

// pub(crate) static SCENE: Lazy<Mutex<Scene>> = Lazy::new(|| Mutex::new(Scene::default()));

pub struct Scene {
    pub name: &'static str,
    pub root_entity_runtime_id: Option<u64>,
    pub handle: shipyard::World,
}

impl Scene {
    pub fn create() -> Arc<Mutex<Scene>> {
        Arc::new(Mutex::new(Self {
            name: "scene",
            handle: shipyard::World::new(),
            root_entity_runtime_id: None,
        }))
    }

    pub fn get_entities_with_component<T: shipyard::Component + Send + Sync + Clone>(
        &self,
    ) -> Vec<u64> {
        let mut entity_id_vec = Vec::new();
        self.handle.run(|vm_view: shipyard::ViewMut<T>| {
            for t in vm_view.iter().with_id() {
                let entity_id = t.0;
                entity_id_vec.push(entity_id);
            }
        });
        let mut entity_vec = Vec::new();
        for entity_id in &entity_id_vec {
            entity_vec.push(entity_id.inner());
        }
        entity_vec
    }

    pub fn get_children_for_entity(scene: Weak<Mutex<Scene>>, entity_id: u64) -> Vec<u64> {
        let entity = Entity::from_handle(entity_id, scene.clone());
        let hierarchy_component: Option<Hierarchy> = entity.get_component();
        let mut result = Vec::new();
        if let Some(hierarchy_component) = hierarchy_component {
            let mut cur_entity_id = hierarchy_component.first_child_runtime_id;
            while let Some(cur_entity_id_unwrapped) = cur_entity_id {
                result.push(cur_entity_id_unwrapped);
                let entity = Entity::from_handle(cur_entity_id_unwrapped, scene.clone());
                let hierarchy_component: Option<Hierarchy> = entity.get_component();
                if let Some(hierarchy_component) = hierarchy_component {
                    cur_entity_id = hierarchy_component.next_sibling_runtime_id;
                } else {
                    cur_entity_id = None;
                }
            }
        }
        result
    }

    pub fn add_child_to_entity(
        scene: Weak<Mutex<Scene>>,
        child_entity_id: u64,
        parent_entity_id: u64,
    ) {
        let child_entity = Entity::from_handle(child_entity_id, scene.clone());
        let parent_entity = Entity::from_handle(parent_entity_id, scene.clone());

        if child_entity.has_component::<Hierarchy>() && parent_entity.has_component::<Hierarchy>() {
            let mut parent_hierarchy_component: Hierarchy = parent_entity.get_component().unwrap();
            let mut child_hierarchy_component: Hierarchy = child_entity.get_component().unwrap();

            if child_hierarchy_component.parent_runtime_id.is_some() {
                // TODO: if child already has parent, remove it from that parent (not a full remove cuz children need to move with it)
                // ^ might be best to create a general 'move' method where you move a child from one parent to a different parent
                todo!();
            }

            parent_hierarchy_component.num_children += 1;
            if parent_hierarchy_component.first_child_runtime_id.is_none() {
                // set child as first child of parent
                parent_hierarchy_component.first_child_runtime_id = Some(child_entity_id);
            } else {
                // insert entity to front of children list
                // set first child of parent to new child
                let former_first_child = parent_hierarchy_component.first_child_runtime_id;
                parent_hierarchy_component.first_child_runtime_id = Some(child_entity_id);
                // set child hierarchy component next to former first child
                child_hierarchy_component.next_sibling_runtime_id = former_first_child;
                // set former first child's previous to this child
                if let Some(former_first_child_entity_id) = former_first_child {
                    let former_first_child_entity =
                        Entity::from_handle(former_first_child_entity_id, scene);
                    if former_first_child_entity.has_component::<Hierarchy>() {
                        let mut former_first_child_hierarchy_component: Hierarchy =
                            former_first_child_entity.get_component().unwrap();
                        former_first_child_hierarchy_component.prev_sibling_runtime_id =
                            Some(child_entity_id);
                        former_first_child_entity
                            .add_component(former_first_child_hierarchy_component);
                    }
                }
            }
            child_hierarchy_component.parent_runtime_id = Some(parent_entity_id);
            parent_entity.add_component(parent_hierarchy_component);
            child_entity.add_component(child_hierarchy_component);
        }
    }

    pub fn create_entity(
        scene: Weak<Mutex<Scene>>,
        name: Option<String>,
        parent_id: Option<u64>,
        transform: Option<Transform>,
    ) -> Result<u64> {
        let scene_mutex = scene.upgrade().ok_or_else(|| {
            anyhow!("Unable to upgrade scene weak reference when creating entity")
        })?;
        let mut scene_mutex_lock = scene_mutex
            .lock()
            .map_err(|_| anyhow!("Unable to acquire scene mutex when creating entity"))
            .unwrap();
        // add root entity if it does not exist
        if scene_mutex_lock.root_entity_runtime_id.is_none() {
            let new_root_entity = scene_mutex_lock
                .handle
                .add_entity((
                    Transform::default(),
                    Hierarchy::default(),
                    Tag::new("Root".into()),
                ))
                .inner();
            scene_mutex_lock.root_entity_runtime_id = Some(new_root_entity);
        }
        // create new entity and make it child of the root
        let new_entity_id = scene_mutex_lock
            .handle
            .add_entity((
                transform.unwrap_or(Transform::default()),
                Hierarchy::default(),
                Tag::new(name.unwrap_or(String::from("Entity"))),
            ))
            .inner();
        let root_id = scene_mutex_lock.root_entity_runtime_id.unwrap();
        // drop mutex lock to allow other threads to modify scene
        drop(scene_mutex_lock);
        Scene::add_child_to_entity(scene, new_entity_id, parent_id.unwrap_or(root_id));
        Ok(new_entity_id)
    }

    pub fn add_gltf_scene(
        scene: Weak<Mutex<Scene>>,
        entity_id: u64,
        resource_manager: &ResourceManager,
        guid: String,
    ) {
        let resource_handle = resource_manager
            .get_resource(guid.clone())
            .expect("Resource handle cannot be found");
        let upgraded_resource_handle = resource_handle
            .upgrade()
            .expect("Unable to upgrade resource handle");
        let path = &upgraded_resource_handle
            .path
            .to_str()
            .expect("Unable to get resource path");
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

        // TODO: apply transformations of gltf_scene to this current entity (with id entity_id)
        for gltf_scene in gltf.scenes() {
            // println!("Scene name: {}", gltf_scene.clone().name().unwrap());
            for node in gltf_scene.nodes() {
                let mut skin_root_nodes = HashSet::new();
                let mut inverse_bind_poses = HashMap::new();
                let mut joint_node_id_to_joint_id = HashMap::new();
                gltf.skins().for_each(|gltf_skin| {
                    match gltf_skin.skeleton() {
                        Some(skeleton) => {
                            skin_root_nodes.insert(skeleton.index() as u32);
                        }
                        None => {
                            // skin_root_nodes.insert(node.index() as u32);
                        }
                    }
                    let reader = gltf_skin.reader(|buffer| Some(&buffer_data[buffer.index()]));
                    let inverse_bindposes: Vec<dream_math::Matrix4<f32>> = reader
                        .read_inverse_bind_matrices()
                        .unwrap()
                        .map(|mat| mat.into())
                        .collect();

                    // how to map inverse bind matrices to joints: https://stackoverflow.com/questions/64904889/what-is-the-correct-mapping-of-inverse-bind-matrices
                    log::debug!("Number of inverse bind poses {:?}", inverse_bindposes.len());
                    log::debug!(
                        "Number of joint for each inverse bind pose {:?}",
                        gltf_skin.joints().len()
                    );

                    // TODO: associate the index of the joint node and its index in the joints array - this is what we should use
                    let mut idx = 0;
                    gltf_skin.joints().for_each(|joint| {
                        // log::debug!(
                        //     "Inverse bind pose for idx {:?} joint {:?} is {:?}",
                        //     idx,
                        //     joint.index(),
                        //     inverse_bindposes[idx]
                        // );
                        joint_node_id_to_joint_id.insert(joint.index() as u32, idx as u32);
                        inverse_bind_poses.insert(joint.index() as u32, inverse_bindposes[idx]);
                        idx += 1;
                    });
                });

                // set transform of root node of GLTF scene to this entity we are adding scene to
                {
                    let transform = get_gltf_transform(&node);
                    let entity = Entity::from_handle(entity_id, scene.clone());
                    entity.add_component(transform);
                }
                let node_idx = &(node.index() as u32);
                process_gltf_child_node(
                    node,
                    &skin_root_nodes,
                    &inverse_bind_poses,
                    &joint_node_id_to_joint_id,
                    scene.clone(),
                    resource_manager,
                    guid.clone(),
                    entity_id,
                    skin_root_nodes.contains(node_idx),
                );
            }
        }

        fn count_number_of_gltf_node_descendents<'a>(child_node: &'a gltf::Node) -> i32 {
            let mut count = 1;
            for child in child_node.children() {
                count += count_number_of_gltf_node_descendents(&child);
            }
            count
        }

        fn collect_gltf_nodes_as_set<'a>(child_node: &'a gltf::Node, nodes: &'a mut HashSet<u32>) {
            nodes.insert(child_node.index() as u32);
            for child in child_node.children() {
                collect_gltf_nodes_as_set(&child, nodes);
            }
        }

        fn process_gltf_child_node(
            child_node: gltf::Node,
            skin_root_nodes: &HashSet<u32>,
            inverse_bind_poses: &HashMap<u32, Matrix4<f32>>,
            joint_node_id_to_joint_id: &HashMap<u32, u32>,
            scene: Weak<Mutex<Scene>>,
            resource_manager: &ResourceManager,
            guid: String,
            entity_id: u64,
            is_bone: bool,
        ) {
            match child_node.mesh() {
                None => {
                    for child in child_node.children() {
                        let new_entity_id = Scene::create_entity(
                            scene.clone(),
                            Some(child.name().unwrap_or("Node").into()),
                            Some(entity_id),
                            Some(get_gltf_transform(&child)),
                        )
                        .expect("Unable to create entity while traversing GLTF nodes");
                        let is_skin_root = skin_root_nodes.contains(&(child.index() as u32));
                        let is_bone = is_bone || is_skin_root;
                        if is_bone {
                            let entity = Entity::from_handle(new_entity_id, scene.clone());
                            entity.add_component(Bone {
                                is_root: is_skin_root,
                                node_id: child.index() as u32,
                                bone_id: *joint_node_id_to_joint_id
                                    .get(&(child.index() as u32))
                                    .unwrap_or(&1),
                                inverse_bind_pose: *inverse_bind_poses
                                    .get(&(child.index() as u32))
                                    .unwrap_or(&Matrix4::<f32>::identity()),
                            });
                        }
                        process_gltf_child_node(
                            child,
                            skin_root_nodes,
                            inverse_bind_poses,
                            joint_node_id_to_joint_id,
                            scene.clone(),
                            resource_manager,
                            guid.clone(),
                            new_entity_id,
                            is_bone,
                        );
                    }
                }
                Some(mesh) => {
                    let new_entity_id = Scene::create_entity(
                        scene.clone(),
                        Some(mesh.name().unwrap_or("Mesh").into()),
                        Some(entity_id),
                        None,
                    )
                    .expect("Unable to create entity while traversing GLTF mesh nodes");
                    MeshRenderer::add_to_entity(
                        scene,
                        new_entity_id,
                        resource_manager,
                        guid,
                        false,
                        Some(mesh.index()),
                    );
                }
            }
        }

        fn get_gltf_transform(node: &gltf::Node) -> Transform {
            let gltf_transform = node.transform();
            let gltf_transform_decomposed = gltf_transform.decomposed();
            let gltf_translation = gltf_transform_decomposed.0;
            let gltf_rotation = gltf_transform_decomposed.1;
            let gltf_scale = gltf_transform_decomposed.2;
            let position = dream_math::Vector3::new(
                gltf_translation[0],
                gltf_translation[1],
                gltf_translation[2],
            );
            let rotation = dream_math::Quaternion::from_parts(
                gltf_rotation[3],
                dream_math::Vector3::new(gltf_rotation[0], gltf_rotation[1], gltf_rotation[2]),
            );
            let scale = dream_math::Vector3::new(gltf_scale[0], gltf_scale[1], gltf_scale[2]);
            Transform::new(position, rotation, scale)
        }
    }
}

pub trait ToEntity {
    fn to_entity(&self, scene: Weak<Mutex<Scene>>) -> Entity;
}

impl ToEntity for u64 {
    fn to_entity(&self, scene: Weak<Mutex<Scene>>) -> Entity {
        Entity::from_handle(*self, scene)
    }
}
