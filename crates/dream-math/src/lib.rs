use std::fmt;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vector3 {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}

impl Vector3 {
    pub fn from(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Position({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<Vector3> for cgmath::Vector3<f32> {
    fn from(vec: Vector3) -> Self {
        cgmath::Vector3::new(vec.x, vec.y, vec.z)
    }
}
