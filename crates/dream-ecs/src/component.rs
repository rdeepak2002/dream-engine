use std::fmt::Debug;
use std::sync::Weak;

use dream_math::Vector3;
use dream_resource::resource_handle::ResourceHandle;

#[derive(shipyard::Component, Debug, Clone, PartialEq, Default)]
pub struct Transform {
    pub position: Vector3,
}

impl Transform {
    pub fn from(position: Vector3) -> Self {
        Self { position }
    }
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transform({})", self.position)
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
}

impl PartialEq for MeshRenderer {
    fn eq(&self, other: &Self) -> bool {
        if self.resource_handle.is_some() && other.resource_handle.is_some() {
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
    pub fn new(resource_handle: Option<Weak<ResourceHandle>>) -> Self {
        Self { resource_handle }
    }
}

// impl Drop for MeshRenderer {
//     fn drop(&mut self) {
//         if self.resource_handle.is_some() {
//             // TODO: globally access resource_manager as singleton and notify it
//             // let rh = self.resource_handle.unwrap();
//             // Arc::strong_count(&rh);
//         }
//     }
// }
