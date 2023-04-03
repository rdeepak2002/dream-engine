use boa_gc::{Finalize, Trace};

use dream_math::Vector3;

#[derive(shipyard::Component, Debug, Clone, Trace, Finalize)]
pub struct Transform {
    pub position: Vector3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(),
        }
    }

    pub fn from(position: Vector3) -> Self {
        Self { position }
    }

    pub fn to_string(&self) -> String {
        format!("Transform({})", self.position.to_string())
    }
}

#[derive(shipyard::Component, Debug, Clone, Trace, Finalize)]
pub struct Hierarchy {
    pub parent_runtime_id: u64,
    pub num_children: usize,
    pub first_child_runtime_id: u64,
    pub prev_sibling_runtime_id: u64,
    pub next_sibling_runtime_id: u64,
}

impl Hierarchy {
    pub fn new() -> Self {
        Self {
            parent_runtime_id: 0,
            num_children: 0,
            first_child_runtime_id: 0,
            prev_sibling_runtime_id: 0,
            next_sibling_runtime_id: 0,
        }
    }
}
