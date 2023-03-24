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
