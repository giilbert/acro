use std::{any::TypeId, cell::UnsafeCell};

use crate::{
    archetype::Archetypes,
    entity::{Entities, EntityId, EntityMeta},
    pointer::change_detection::{ChangeDetectionContext, Tick},
    query::{Query, QueryFilter, ToQueryInfo},
    registry::{ComponentInfo, ComponentRegistry},
    resource::ResourceRegistry,
    systems::IntoSystem,
};

#[derive(Debug)]
pub struct World {
    components: ComponentRegistry,
    entities: Entities,
    pub(crate) resources: ResourceRegistry,
    pub(crate) archetypes: Archetypes,
}

impl World {
    pub fn new() -> Self {
        Self {
            components: ComponentRegistry::new(),
            entities: Entities::new(),
            resources: ResourceRegistry::new(),
            archetypes: Archetypes::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        self.archetypes.push_empty_entity(&mut self.entities)
    }

    pub fn entity_meta(&self, entity: EntityId) -> &EntityMeta {
        self.entities.get(entity).expect("entity not found")
    }

    pub fn init_component<T: 'static>(&mut self) -> &ComponentInfo {
        self.components.init_rust_type::<T>()
    }

    pub fn get_component_info<T: 'static>(&self) -> &ComponentInfo {
        self.components.get::<T>().expect("component not found")
    }

    pub fn get_component_info_id(&self, id: TypeId) -> &ComponentInfo {
        self.components.get_by_id(id).expect("component not found")
    }

    pub fn insert<T: 'static>(&mut self, entity: EntityId, component: T) {
        let component_info = self.components.get::<T>().expect("component not found");
        self.archetypes.add_component(
            &self.components,
            &mut self.entities,
            entity,
            component_info.id,
            component,
        );
    }

    pub fn remove<T: 'static>(&mut self, entity: EntityId) -> T {
        let component_info = self.components.get::<T>().expect("component not found");
        self.archetypes.remove_component(
            &self.components,
            &mut self.entities,
            entity,
            component_info.id,
        )
    }

    pub fn query<T, F>(&mut self) -> Query<T, F>
    where
        T: ToQueryInfo,
        F: QueryFilter,
    {
        Query::<T, F>::new(self)
    }

    pub fn run_system<I, P>(&mut self, system: I, tick: Tick)
    where
        I: IntoSystem<P>,
        P: 'static,
    {
        let mut system_init = I::init(&self);
        let system_run_function = system.into_system();
        (system_run_function)(self, tick, system_init.as_mut());
    }

    pub fn get<T: 'static>(&self, entity: EntityId) -> Option<&T> {
        let component_info = self.components.get::<T>()?;
        self.archetypes
            .get_component::<T>(&self.entities, entity, component_info.id)
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(resource);
    }
}

#[cfg(test)]
mod tests {
    use crate::archetype::ArchetypeId;

    use super::*;

    #[test]
    fn archetype_creation() {
        let mut world = World::new();
        world.init_component::<u32>();

        let entity_1 = world.spawn();
        let entity_2 = world.spawn();

        let entity_meta_1 = world.entity_meta(entity_1);
        let entity_meta_2 = world.entity_meta(entity_2);
        assert_eq!(entity_meta_2.archetype_id, ArchetypeId(0));
        assert_eq!(entity_meta_1.table_index, 0);
        assert_eq!(entity_meta_2.table_index, 1);
        world.insert(entity_1, 42u32);

        let entity_meta_1_after_move = world.entity_meta(entity_1);
        assert_eq!(entity_meta_1_after_move.archetype_id, ArchetypeId(1));
        assert_eq!(entity_meta_1_after_move.table_index, 0);
        let entity_meta_2_after_move = world.entity_meta(entity_2);
        assert_eq!(entity_meta_2_after_move.archetype_id, ArchetypeId(0));
        assert_eq!(entity_meta_2_after_move.table_index, 0);
    }

    #[test]
    fn component_removal() {
        let mut world = World::new();
        world.init_component::<u32>();

        let entity_1 = world.spawn();
        let entity_meta_1 = world.entity_meta(entity_1);
        assert_eq!(entity_meta_1.archetype_id, ArchetypeId::EMPTY);

        world.insert(entity_1, 42u32);

        let entity_meta_1 = world.entity_meta(entity_1);
        assert_eq!(entity_meta_1.archetype_id, ArchetypeId(1));

        let data = world.remove::<u32>(entity_1);
        assert_eq!(data, 42);

        let entity_meta_1 = world.entity_meta(entity_1);
        assert_eq!(entity_meta_1.archetype_id, ArchetypeId::EMPTY);
    }
}
