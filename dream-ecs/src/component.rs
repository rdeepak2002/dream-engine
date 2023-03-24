#[derive(shipyard::Component, Debug, Clone)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_string(&self) -> String {
        format!("Transform({}, {}, {})", self.x, self.y, self.z)
    }
}
