use std::{cell::RefCell, collections::HashMap, ptr::NonNull};

use crate::{
    entity::{Entities, EntityId, EntityMeta},
    registry::{ComponentGroup, ComponentId, ComponentRegistry},
    storage::table::Table,
};

#[derive(Debug)]
pub struct Archetypes {
    current_id: usize,
    archetypes: HashMap<ArchetypeId, RefCell<Archetype>>,
    components: HashMap<ComponentGroup, ArchetypeId>,
    // Maps from an old archetype to a set of new archetypes based on components added or removed
    edges: Edges,
}

#[derive(Debug, Clone, Copy)]
enum ArchetypeOperation {
    Insert,
    Remove,
}

impl Archetypes {
    pub fn new() -> Self {
        let mut archetypes = HashMap::new();
        let mut components = HashMap::new();

        // Create the archetype with no components
        archetypes.insert(
            ArchetypeId(0),
            RefCell::new(Archetype::new(ComponentGroup::new(vec![]), ArchetypeId(0))),
        );
        components.insert(ComponentGroup::new(vec![]), ArchetypeId(0));

        Self {
            // Skip ArchetypeId(0) because it's reserved for ArchetypeId::NONE
            current_id: 1,
            archetypes,
            components,
            edges: Edges::new(),
        }
    }

    fn new_archetype(&mut self, components: ComponentGroup) -> ArchetypeId {
        let id = ArchetypeId(self.current_id);
        self.edges.init_archetype(id);
        self.current_id += 1;

        let archetype = Archetype::new(components.clone(), id);
        self.archetypes.insert(id, RefCell::new(archetype));
        self.components.insert(components, id);

        id
    }

    #[inline]
    fn get_or_create_archetype(
        &mut self,
        current_archetype: ArchetypeId,
        new_component: ComponentId,
        operation: ArchetypeOperation,
        component_registry: &ComponentRegistry,
    ) -> ArchetypeId {
        match self.edges.get(current_archetype, operation, new_component) {
            Some(id) => id,
            None => {
                let new_archetype_components = {
                    let old_components = &self.archetypes[&current_archetype].borrow().components;
                    let new_component_info = component_registry.get_info(new_component).clone();
                    match operation {
                        ArchetypeOperation::Insert => old_components.extend(new_component_info),
                        ArchetypeOperation::Remove => old_components.remove(new_component_info),
                    }
                };

                match self.components.get(&new_archetype_components).cloned() {
                    Some(id) => {
                        self.edges
                            .create_insert_edge(current_archetype, new_component, id);
                        self.edges
                            .create_remove_edge(id, new_component, current_archetype);
                        id
                    }
                    None => self.new_archetype(new_archetype_components),
                }
            }
        }
    }

    #[inline]
    fn move_entity(
        &self,
        id: EntityId,
        from: &RefCell<Archetype>,
        to: &RefCell<Archetype>,
        entities: &mut Entities,
        new_component_data: impl Iterator<Item = (ComponentId, *const u8)>,
    ) {
        let current_meta = entities.get_mut(id).expect("entity not found");
        let moved_last_entity = from.borrow_mut().remove(current_meta);
        unsafe {
            to.borrow_mut().copy_entity_with_components(
                id,
                current_meta,
                &*from.borrow(),
                new_component_data,
            );
        };

        current_meta.archetype_id = to.borrow().id;
        let old_entity_table_index = current_meta.table_index;

        if let Some(entity_id) = moved_last_entity {
            let moved_last_entity_meta = entities.get_mut(entity_id).expect("entity not found");
            moved_last_entity_meta.table_index = old_entity_table_index;
        }

        entities.get_mut(id).unwrap().table_index = to.borrow().entities.len() - 1;
    }

    pub fn add_component<T: 'static>(
        &mut self,
        component_registry: &ComponentRegistry,
        entities: &mut Entities,
        entity: EntityId,
        new_component: ComponentId,
        data: T,
    ) {
        let meta = entities.get_mut(entity).expect("entity not found");
        let new_archetype_id = self.get_or_create_archetype(
            meta.archetype_id,
            new_component,
            ArchetypeOperation::Insert,
            component_registry,
        );

        let new_archetype = &self.archetypes[&new_archetype_id];
        let old_archetype = &self.archetypes[&meta.archetype_id];

        self.move_entity(
            entity,
            old_archetype,
            new_archetype,
            entities,
            [(new_component, &data as *const T as *const u8)].into_iter(),
        );
    }

    pub fn remove_component<T: 'static>(
        &mut self,
        component_registry: &ComponentRegistry,
        entities: &mut Entities,
        entity: EntityId,
        remove_component: ComponentId,
    ) -> T {
        let meta = entities.get_mut(entity).expect("entity not found");
        let new_archetype_id = self.get_or_create_archetype(
            meta.archetype_id,
            remove_component,
            ArchetypeOperation::Remove,
            component_registry,
        );

        let new_archetype = &self.archetypes[&new_archetype_id];
        let old_archetype = &self.archetypes[&meta.archetype_id];

        let removed_component_data = old_archetype
            .borrow_mut()
            .pointer_to_entity_component(&meta, remove_component)
            .expect("component data not found")
            .as_ptr() as *const T;

        self.move_entity(
            entity,
            old_archetype,
            new_archetype,
            entities,
            std::iter::empty(),
        );

        unsafe { removed_component_data.read() }
    }

    pub fn push_empty_entity(&mut self, entities: &mut Entities) -> EntityId {
        let mut empty_archetype = self.archetypes[&ArchetypeId::EMPTY].borrow_mut();
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
        self.table.columns[&component].get_ptr(entity_meta.table_index)
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
    pub unsafe fn copy_entity_with_components(
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

#[derive(Debug, Default)]
struct Edges {
    insert_edges: HashMap<ArchetypeId, HashMap<ComponentId, ArchetypeId>>,
    remove_edges: HashMap<ArchetypeId, HashMap<ComponentId, ArchetypeId>>,
}

impl Edges {
    pub fn new() -> Self {
        let mut insert_edges = HashMap::new();
        let mut remove_edges = HashMap::new();

        insert_edges.insert(ArchetypeId::EMPTY, HashMap::new());
        remove_edges.insert(ArchetypeId::EMPTY, HashMap::new());

        Self {
            insert_edges,
            remove_edges,
        }
    }

    #[inline]
    pub fn get(
        &mut self,
        archetype_id: ArchetypeId,
        operation: ArchetypeOperation,
        component_id: ComponentId,
    ) -> Option<ArchetypeId> {
        let edges = match operation {
            ArchetypeOperation::Insert => &self.insert_edges,
            ArchetypeOperation::Remove => &self.remove_edges,
        };
        edges[&archetype_id].get(&component_id).cloned()
    }

    #[inline]
    pub fn create_insert_edge(
        &mut self,
        archetype_id: ArchetypeId,
        component_id: ComponentId,
        new_archetype_id: ArchetypeId,
    ) {
        self.insert_edges
            .entry(archetype_id)
            .or_insert_with(HashMap::new)
            .insert(component_id, new_archetype_id);
    }

    #[inline]
    pub fn create_remove_edge(
        &mut self,
        archetype_id: ArchetypeId,
        component_id: ComponentId,
        new_archetype_id: ArchetypeId,
    ) {
        self.remove_edges
            .entry(archetype_id)
            .or_insert_with(HashMap::new)
            .insert(component_id, new_archetype_id);
    }

    pub fn init_archetype(&mut self, archetype_id: ArchetypeId) {
        self.insert_edges.insert(archetype_id, HashMap::new());
        self.remove_edges.insert(archetype_id, HashMap::new());
    }
}
