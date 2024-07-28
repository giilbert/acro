use nalgebra as na;

cfg_if::cfg_if! {
    if #[cfg(feature = "double-precision")] {
        pub type Float = f64;
    } else {
        pub type Float = f32;
    }
}

pub type Vec2 = na::Vector2<Float>;
pub type Vec3 = na::Vector3<Float>;
pub type Vec4 = na::Vector4<Float>;

pub type Mat2 = na::Matrix2<Float>;
pub type Mat3 = na::Matrix3<Float>;
pub type Mat4 = na::Matrix4<Float>;

pub type UnitQuaternion = na::UnitQuaternion<Float>;
