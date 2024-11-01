use acro_math::{Float, Vec3};

pub struct Rigidbody3D;

pub struct Mass(pub Float);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vec3);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Force(pub Vec3);
