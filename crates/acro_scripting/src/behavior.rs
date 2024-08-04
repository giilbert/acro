use acro_assets::Asset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Behavior {
    pub source: String,
    #[serde(skip)]
    pub(crate) data: Option<BehaviorData>,
}

#[derive(Debug)]
pub struct BehaviorData {
    pub(crate) id: u32,
}

impl Behavior {
    pub fn new(source_file_path: impl ToString) -> Self {
        Self {
            source: source_file_path.to_string(),
            data: None,
        }
    }
}
