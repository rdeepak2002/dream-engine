use nalgebra::{Rotation3, UnitQuaternion};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Quaternion {
    pub vector: Vector3<f32>,
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
    pub fn new(vector: Vector3<f32>, scalar: f32) -> Self {
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
        Self {
            vector: quaternion.vector().xyz(),
            scalar: quaternion.scalar(),
        }
    }
}

pub type Vector3<T> = nalgebra::Vector3<T>;
