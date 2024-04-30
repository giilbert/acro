use std::{cell::RefCell, collections::HashMap, ptr::NonNull};

use crate::{
    entity::{Entities, EntityId, EntityMeta},
    registry::{ComponentGroup, ComponentId, ComponentRegistry},
    storage::table::Table,
};

#[derive(Debug)]
pub struct Archetypes {
    current_id: usize,
    data: HashMap<ArchetypeId, RefCell<Archetype>>,
    components: HashMap<ComponentGroup, ArchetypeId>,
    // Maps from an old archetype to a set of new archetypes based on components added or removed
    edges: HashMap<ArchetypeId, HashMap<ArchetypeOperation, ArchetypeId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ArchetypeOperation {
    AddComponent(ComponentId),
    // TODO: remove component
}

impl Archetypes {
    pub fn new() -> Self {
        let mut data = HashMap::new();

        // Create the archetype with no components
        data.insert(
            ArchetypeId(0),
            RefCell::new(Archetype::new(ComponentGroup::new(vec![]), ArchetypeId(0))),
        );

        Self {
            // Skip ArchetypeId(0) because it's reserved for ArchetypeId::NONE
            current_id: 1,
            data,
            components: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    fn new_archetype(&mut self, components: ComponentGroup) -> ArchetypeId {
        let id = ArchetypeId(self.current_id);
        self.current_id += 1;

        let archetype = Archetype::new(components.clone(), id);
        self.data.insert(id, RefCell::new(archetype));
        self.components.insert(components, id);

        id
    }

    pub fn add_component<T: 'static>(
        &mut self,
        component_registry: &ComponentRegistry,
        entities: &mut Entities,
        entity: EntityId,
        new_component: ComponentId,
        data: T,
    ) {
        let entity_meta = entities.get_mut(entity).expect("entity not found");
        let edges = self.edges.entry(entity_meta.archetype_id).or_default();
        let possible_edge = edges
            .get(&ArchetypeOperation::AddComponent(new_component))
            .cloned();

        let new_archetype_id = match possible_edge {
            Some(id) => id,
            None => {
                let (&old_archetype_id, old_archetype) = self
                    .data
                    .get_key_value(&entity_meta.archetype_id)
                    .expect("entity's current archetype not found");

                let new_component_info = component_registry
                    .get_info(new_component)
                    .expect("component not found");
                let new_archetype_components = old_archetype
                    .borrow_mut()
                    .components
                    .extend(new_component_info.clone());

                match self.components.get(&new_archetype_components) {
                    Some(&id) => id,
                    None => {
                        let new_archetype_id = self.new_archetype(new_archetype_components);
                        self.edges
                            .get_mut(&old_archetype_id)
                            .expect("edges could not be found")
                            .insert(
                                ArchetypeOperation::AddComponent(new_component),
                                new_archetype_id,
                            );
                        new_archetype_id
                    }
                }
            }
        };

        let new_archetype = &self.data[&new_archetype_id];
        let old_archetype = &self.data[&entity_meta.archetype_id];

        let moved_last_entity = old_archetype.borrow_mut().remove(entity_meta);
        unsafe {
            new_archetype.borrow_mut().copy_entity_add_components(
                entity,
                entity_meta,
                &*old_archetype.borrow(),
                [(new_component, &data as *const T as *const u8)].into_iter(),
            );
        };

        entity_meta.archetype_id = new_archetype_id;
        let old_entity_table_index = entity_meta.table_index;

        if let Some(entity_id) = moved_last_entity {
            let moved_last_entity_meta = entities.get_mut(entity_id).expect("entity not found");
            moved_last_entity_meta.table_index = old_entity_table_index;
            println!("moved last entity");
        }

        entities.get_mut(entity).unwrap().table_index = new_archetype.borrow().entities.len() - 1;
    }

    pub fn push_empty_entity(&mut self, entities: &mut Entities) -> EntityId {
        let mut empty_archetype = self.data[&ArchetypeId::EMPTY].borrow_mut();
        let new_entity_table_index = empty_archetype.entities.len();
        let entity = entities.spawn(new_entity_table_index);
        empty_archetype.entities.push(entity);
        unsafe { empty_archetype.table.push_row([].into_iter()) };
        entity
    }
}

#[derive(Debug)]
pub struct Archetype {
    id: ArchetypeId,
    table: Table,
    components: ComponentGroup,
    entities: Vec<EntityId>,
}

impl Archetype {
    pub fn new(components: ComponentGroup, id: ArchetypeId) -> Self {
        let table = Table::new(components.clone());
        Archetype {
            id,
            table,
            components,
            entities: vec![],
        }
    }

    #[inline]
    pub fn pointer_to_entity_component(
        &self,
        entity_meta: &EntityMeta,
        component: ComponentId,
    ) -> Option<NonNull<u8>> {
        self.table.columns[&component].get_ptr(entity_meta.archetype_id.0)
    }

    pub fn remove(&mut self, entity_meta: &EntityMeta) -> Option<EntityId> {
        let last_affected = self
            .table
            .remove_row(entity_meta.table_index)
            .map(|index| self.entities.get(index).cloned())
            .flatten();
        self.entities.swap_remove(entity_meta.table_index);
        last_affected
    }

    /// Moves an entity's data from `old_archetype` to this archetype where the difference is the addition
    /// of components.
    pub unsafe fn copy_entity_add_components(
        &mut self,
        entity_id: EntityId,
        entity_meta: &EntityMeta,
        old_archetype: &Archetype,
        new_component_data: impl Iterator<Item = (ComponentId, *const u8)>,
    ) {
        self.table.push_row(
            new_component_data.chain(old_archetype.components.iter().filter_map(|info| {
                if self.components.contains(info.id) {
                    None
                } else {
                    old_archetype
                        .pointer_to_entity_component(entity_meta, info.id)
                        .map(|ptr| (info.id, ptr.as_ptr() as *const u8))
                }
            })),
        );
        self.entities.push(entity_id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub usize);

impl ArchetypeId {
    pub const EMPTY: Self = ArchetypeId(0);
}
