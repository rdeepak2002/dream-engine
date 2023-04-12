// use boa_gc::{Finalize, Trace};

// #[derive(Debug, Clone, Trace, Finalize)]
#[derive(Debug, Clone, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn from(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_string(&self) -> String {
        format!("Position({}, {}, {})", self.x, self.y, self.z)
    }

    pub fn to_cg_math(&self) -> cgmath::Vector3<f32> {
        return cgmath::Vector3::new(self.x, self.y, self.z);
    }
}
