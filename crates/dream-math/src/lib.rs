use std::fmt;

use nalgebra::{Rotation3, UnitQuaternion};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Quaternion {
    pub vector: Vector3,
    pub scalar: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            vector: Vector3::default(),
            scalar: 1.0,
        }
    }
}

impl Quaternion {
    pub fn new(vector: Vector3, scalar: f32) -> Self {
        Self { vector, scalar }
    }

    pub fn from_xyz_euler_angles_degrees(x: f32, y: f32, z: f32) -> Self {
        let quaternion: UnitQuaternion<f32> = Rotation3::from_euler_angles(x, y, z).into();
        let vec = quaternion.vector();
        Self {
            vector: Vector3::new(vec.x, vec.y, vec.z),
            scalar: quaternion.scalar(),
        }
    }
}

impl From<Quaternion> for nalgebra::Quaternion<f32> {
    fn from(quaternion: Quaternion) -> Self {
        nalgebra::Quaternion::from_parts(
            quaternion.scalar,
            nalgebra::Vector3::from(quaternion.vector),
        )
    }
}

impl From<nalgebra::Quaternion<f32>> for Quaternion {
    fn from(quaternion: nalgebra::Quaternion<f32>) -> Self {
        let vec = quaternion.vector();
        Self {
            vector: Vector3::new(vec.x, vec.y, vec.z),
            scalar: quaternion.scalar(),
        }
    }
}

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
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Position({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<Vector3> for nalgebra::Vector3<f32> {
    fn from(vec: Vector3) -> Self {
        nalgebra::Vector3::new(vec.x, vec.y, vec.z)
    }
}

impl From<nalgebra::Vector3<f32>> for Vector3 {
    fn from(vec: nalgebra::Vector3<f32>) -> Self {
        Vector3::new(vec.x, vec.y, vec.z)
    }
}

impl std::ops::Add<Vector3> for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Vector3) -> Vector3 {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Vector3 {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul<Vector3> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl std::ops::Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f32) -> Vector3 {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Mul<Vector3> for f32 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}
