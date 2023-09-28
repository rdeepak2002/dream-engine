use std::fmt::Debug;
use std::sync::{Mutex, Weak};

use dream_math::{Quaternion, Vector3};
use dream_resource::resource_handle::ResourceHandle;
use dream_resource::resource_manager::ResourceManager;

use crate::entity::Entity;
use crate::scene::Scene;

#[derive(shipyard::Component, Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::default(),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transform({})", self.position)
    }
}

#[derive(shipyard::Component, Default, Debug, Clone, PartialEq)]
pub struct Light {
    pub color: Vector3<f32>,
}

impl Light {
    pub fn new(color: Vector3<f32>) -> Light {
        Light { color }
    }
}

impl std::fmt::Display for Light {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Light({})", self.color)
    }
}

#[derive(shipyard::Component, Default, Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tag({})", self.name)
    }
}

// TODO: when serializing this, we don't need to create a guid field cuz
// when deserializing we can create a temporary map that maps <old runtime id: new runtime id>
#[derive(shipyard::Component, Default, Debug, Clone, PartialEq)]
pub struct Hierarchy {
    pub num_children: usize,
    pub parent_runtime_id: Option<u64>,
    pub first_child_runtime_id: Option<u64>,
    pub prev_sibling_runtime_id: Option<u64>,
    pub next_sibling_runtime_id: Option<u64>,
}
#[derive(shipyard::Component, Debug, Clone, Default)]
pub struct MeshRenderer {
    pub resource_handle: Option<Weak<ResourceHandle>>,
    pub mesh_idx: Option<usize>,
}

impl PartialEq for MeshRenderer {
    fn eq(&self, other: &Self) -> bool {
        if self.mesh_idx == other.mesh_idx
            && self.resource_handle.is_some()
            && other.resource_handle.is_some()
        {
            let r1 = self.resource_handle.as_ref().unwrap().upgrade();
            let r2 = other.resource_handle.as_ref().unwrap().upgrade();
            if r1.is_some() && r2.is_some() {
                return r1.unwrap().eq(&r2.unwrap());
            }
        }
        false
    }
}

impl MeshRenderer {
    pub fn new(resource_handle: Option<Weak<ResourceHandle>>, mesh_idx: Option<usize>) -> Self {
        Self {
            resource_handle,
            mesh_idx,
        }
    }

    pub fn add_to_entity(
        scene: Weak<Mutex<Scene>>,
        entity_handle: u64,
        resource_manager: &ResourceManager,
        guid: String,
        create_child_nodes: bool,
        mesh_idx: Option<usize>,
    ) {
        let resource_handle = resource_manager
            .get_resource(guid.clone())
            .expect("Resource handle cannot be found");
        Entity::from_handle(entity_handle, scene.clone())
            .add_component(MeshRenderer::new(Some(resource_handle), mesh_idx));
        if create_child_nodes {
            Scene::add_gltf_scene(scene, entity_handle, resource_manager, guid);
        }
    }
}

#[derive(shipyard::Component, Debug, Clone, Default)]
pub struct PythonScript {
    pub resource_handle: Option<Weak<ResourceHandle>>,
}

impl PythonScript {
    pub fn new(resource_handle: Option<Weak<ResourceHandle>>) -> Self {
        Self { resource_handle }
    }

    pub fn add_to_entity(
        scene: Weak<Mutex<Scene>>,
        entity_handle: u64,
        resource_manager: &ResourceManager,
        guid: String,
    ) {
        let resource_handle = resource_manager
            .get_resource(guid)
            .expect("Resource handle cannot be found");
        Entity::from_handle(entity_handle, scene)
            .add_component(PythonScript::new(Some(resource_handle)));
    }
}
