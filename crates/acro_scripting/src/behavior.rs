use acro_assets::Asset;

use crate::source_file::SourceFile;

#[derive(Debug)]
pub struct Behavior {
    pub(crate) source_file_path: String,
    pub(crate) data: Option<BehaviorData>,
}

#[derive(Debug)]
pub struct BehaviorData {
    pub(crate) id: u32,
}

impl Behavior {
    pub fn new(source_file_path: impl ToString) -> Self {
        Self {
            source_file_path: source_file_path.to_string(),
            data: None,
        }
    }
}
