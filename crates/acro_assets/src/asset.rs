use std::{
    any::Any,
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use acro_ecs::{ComponentId, EntityId, SystemRunContext};
use parking_lot::RwLock;

use crate::loader::Loadable;

#[derive(Debug)]
pub struct Asset<T: Loadable> {
    pub(crate) data: Arc<T>,
    pub(crate) notify_changes: Arc<RwLock<HashMap<EntityId, HashSet<ComponentId>>>>,
}

impl<T: Loadable> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            notify_changes: Arc::clone(&self.notify_changes),
        }
    }
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
    pub fn notify_changes<C: 'static>(&self, ctx: &SystemRunContext, entity_id: EntityId) {
        let mut changes = self.notify_changes.write();
        let entry = changes.entry(entity_id).or_default();
        entry.insert(ctx.world.get_component_info::<C>().id);
    }

    pub fn remove_notify_changes_from_entities(&self, entity_id: EntityId) {
        self.notify_changes.write().remove(&entity_id);
    }

    pub fn remove_notify_changes_from_component(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) {
        if let Some(changes) = self.notify_changes.write().get_mut(&entity_id) {
            changes.remove(&component_id);
        }
    }
}
