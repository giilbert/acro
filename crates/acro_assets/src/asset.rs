use std::{any::Any, ops::Deref, sync::Arc};

use acro_ecs::{EntityId, SystemRunContext};

use crate::loader::Loadable;

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

impl<T> Asset<T>
where
    T: Loadable,
{
    // TODO: Implement this
    pub fn notify_changes(&self, entity_id: EntityId, ctx: &SystemRunContext) {}
}
