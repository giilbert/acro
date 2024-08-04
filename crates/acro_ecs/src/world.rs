use std::{
    any::TypeId,
    cell::{RefCell, UnsafeCell},
    ptr::NonNull,
};

use crate::{
    archetype::Archetypes,
    bundle::Bundle,
    entity::{Entities, EntityId, EntityMeta},
    pointer::change_detection::{ChangeDetectionContext, Tick},
    query::{Query, QueryFilter, ToQueryInfo},
    registry::{ComponentInfo, ComponentRegistry},
    resource::ResourceRegistry,
    systems::{IntoSystem, SystemRunContext},
    ComponentId, Res, ResMut,
};

pub struct World {
    components: ComponentRegistry,
    entities: Entities,
    pub(crate) resources: ResourceRegistry,
    pub(crate) archetypes: Archetypes,
    pub(crate) swap_fn: RefCell<Option<Box<dyn FnOnce(&mut World)>>>,
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("components", &self.components)
            .field("entities", &self.entities)
            .field("resources", &self.resources)
            .field("archetypes", &self.archetypes)
            .field("swap_fn", &"...")
            .finish()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            components: ComponentRegistry::new(),
            entities: Entities::new(),
            resources: ResourceRegistry::new(),
            archetypes: Archetypes::new(),
            swap_fn: RefCell::new(None),
        }
    }

    pub fn spawn<T: Bundle>(&mut self, bundle: T) -> EntityId {
        let entity = self.spawn_empty();
        bundle.build(self, entity);
        entity
    }

    pub fn spawn_empty(&mut self) -> EntityId {
        self.archetypes.push_empty_entity(&mut self.entities)
    }

    pub fn resources(&self) -> &ResourceRegistry {
        &self.resources
    }

    pub fn entity_meta_opt(&self, entity: EntityId) -> Option<&EntityMeta> {
        self.entities.get(entity)
    }

    pub fn entity_meta(&self, entity: EntityId) -> &EntityMeta {
        self.entity_meta_opt(entity).expect("entity not found")
    }

    pub fn init_component<T: 'static>(&mut self) -> &ComponentInfo {
        self.components.init_rust_type::<T>()
    }

    pub fn get_component_info<T: 'static>(&self) -> &ComponentInfo {
        self.components
            .get::<T>()
            .unwrap_or_else(|| panic!("component {} not found", std::any::type_name::<T>()))
    }

    pub fn get_component_info_id(&self, id: TypeId) -> &ComponentInfo {
        self.components
            .get_by_id(id)
            .unwrap_or_else(|| panic!("component with id {id:?} not found"))
    }

    pub fn insert<T: 'static>(&mut self, entity: EntityId, component: T) {
        let component_info = self
            .components
            .get::<T>()
            .unwrap_or_else(|| panic!("component {} not found", std::any::type_name::<T>()));

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
        F: QueryFilter + 'static,
    {
        Query::<T, F>::new(self)
    }

    pub fn run_system<I, P>(&mut self, system: I, tick: Tick) -> eyre::Result<()>
    where
        I: IntoSystem<P>,
        P: 'static,
    {
        let mut system_init = I::init(&self);
        let system_run_function = system.into_system();
        (system_run_function)(
            SystemRunContext {
                world: self,
                tick,
                last_run_tick: Tick::new(0),
            },
            system_init.as_mut(),
        )
    }

    pub fn get<T: 'static>(&self, entity: EntityId) -> Option<&T> {
        let component_info = self.components.get::<T>()?;
        self.archetypes
            .get_component::<T>(&self.entities, entity, component_info.id)
    }

    pub fn get_ptr(
        &self,
        entity: EntityId,
        component_id: ComponentId,
        update_change_detection: Option<Tick>,
    ) -> Option<NonNull<u8>> {
        let component_info = self.components.get_info(component_id);
        self.archetypes.get_component_untyped(
            &self.entities,
            entity,
            component_info.id,
            update_change_detection,
        )
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    pub fn resource<T: 'static>(&self) -> Res<T> {
        self.resources.get::<T>()
    }

    pub fn resource_mut<T: 'static>(&self) -> ResMut<T> {
        self.resources.get_mut::<T>()
    }

    pub fn clear_all_entities(&mut self) {
        self.entities.clear();
        self.archetypes.clear();
    }

    pub fn queue_swap(&self, swapper: impl FnOnce(&mut World) + 'static) {
        *self.swap_fn.borrow_mut() = Some(Box::new(swapper));
    }

    pub fn check_swap(&mut self) {
        if let Some(swap_fn) = self.swap_fn.replace(None) {
            swap_fn(self);
        }
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

        let entity_1 = world.spawn_empty();
        let entity_2 = world.spawn_empty();

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

        let entity_1 = world.spawn_empty();
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
