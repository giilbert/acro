use acro_render::Color;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Panel {
    pub(crate) color: Color,
}

impl Panel {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}
