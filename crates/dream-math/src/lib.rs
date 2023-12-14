pub type Quaternion<T> = nalgebra::Quaternion<T>;
pub type UnitQuaternion<T> = nalgebra::UnitQuaternion<T>;
pub type Vector2<T> = nalgebra::Vector2<T>;
pub type Vector3<T> = nalgebra::Vector3<T>;
pub type Vector4<T> = nalgebra::Vector4<T>;
pub type UnitVector3<T> = nalgebra::UnitVector3<T>;
pub type Point3<T> = nalgebra::Point3<T>;
pub type Point4<T> = nalgebra::Point4<T>;
pub type Matrix4<T> = nalgebra::Matrix4<T>;
pub type Rotation3<T> = nalgebra::Rotation3<T>;

pub fn pi() -> f32 {
    nalgebra::RealField::pi()
}

pub fn radians(degrees: f32) -> f32 {
    degrees * pi() / 180.0
}

pub fn degrees(radians: f32) -> f32 {
    radians * 180.0 / pi()
}

#[macro_export]
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

#[macro_export]
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}
