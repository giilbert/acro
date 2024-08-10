use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Color {
    Srgba(Srgba),
}

impl Color {
    pub fn to_srgba(&self) -> Srgba {
        match self {
            Color::Srgba(srgba) => *srgba,
        }
    }
}

// All of the fields are [0.0, 1.0]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct Srgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Srgba {
    pub const WHITE: Srgba = Srgba::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Srgba = Srgba::new(0.0, 0.0, 0.0, 1.0);

    pub const RED: Srgba = Srgba::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Srgba = Srgba::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Srgba = Srgba::new(0.0, 0.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}
