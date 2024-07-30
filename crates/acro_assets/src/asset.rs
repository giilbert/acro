use std::{any::Any, ops::Deref, sync::Arc};

use crate::loader::Loadable;

pub type AnyAssetData = Arc<dyn Any + Send + Sync>;

#[derive(Debug)]
pub struct Asset<T: Loadable> {
    pub(crate) data: Arc<T>,
}

impl<T: Loadable> Deref for Asset<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}
