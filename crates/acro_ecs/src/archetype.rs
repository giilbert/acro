use std::{
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
    ptr::NonNull,
    rc::Rc,
};

use crate::{
    entity::{Entities, EntityId, EntityMeta},
    pointer::change_detection::{ChangeDetectionContext, Tick},
    registry::{ComponentGroup, ComponentId, ComponentRegistry},
    storage::{anyvec::AnyVec, table::Table},
};

#[derive(Debug)]
pub struct Archetypes {
    pub(crate) generation: usize,
    current_id: usize,
    /// The generation is used to know if query archetype ids needs to recomputed
    archetypes: HashMap<ArchetypeId, RefCell<Archetype>>,
    components: HashMap<ComponentGroup, ArchetypeId>,
    // Maps from an old archetype to a set of new archetypes based on components added or removed
    pub(crate) edges: Edges,
}

#[derive(Debug, Clone, Copy)]
pub enum ArchetypeOperation {
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
            generation: 0,
            archetypes,
            components,
            edges: Edges::new(),
        }
    }

    fn new_archetype(&mut self, components: ComponentGroup) -> ArchetypeId {
        self.generation += 1;

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
                self.generation += 1;

                let new_archetype_components = {
                    let old_components = &self.archetypes[&current_archetype].borrow().components;
                    let new_component_info = component_registry.get_info(new_component).clone();
                    match operation {
                        ArchetypeOperation::Insert => old_components.extend(new_component_info),
                        ArchetypeOperation::Remove => old_components.remove(new_component_info),
                    }
                };

                let id = match self.components.get(&new_archetype_components).cloned() {
                    Some(id) => id,
                    None => self.new_archetype(new_archetype_components.clone()),
                };

                id
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

        unsafe {
            to.borrow_mut().copy_entity_with_components(
                id,
                current_meta,
                &*from.borrow(),
                new_component_data,
            );
        };

        let moved_last_entity = from.borrow_mut().remove(current_meta);

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

        std::mem::forget(data);
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
            .pointer_to_entity_component(meta.table_index, remove_component)
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

    pub fn get_archetype(&self, id: ArchetypeId) -> Option<&RefCell<Archetype>> {
        self.archetypes.get(&id)
    }

    pub fn get_archetypes_with<'a>(&'a self, components: &'a ComponentGroup) -> Vec<ArchetypeId> {
        self.archetypes
            .values()
            .filter_map(|archetype| {
                let archetype = archetype.borrow();
                if components.is_subset_of(&archetype.components) {
                    Some(archetype.id)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Archetype {
    pub(crate) id: ArchetypeId,
    pub(crate) table: Table,
    pub(crate) components: ComponentGroup,
    pub(crate) entities: Vec<EntityId>,
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
        table_index: usize,
        component: ComponentId,
    ) -> Option<NonNull<u8>> {
        unsafe { (&*self.table.columns[&component].data.get() as &AnyVec).get_ptr(table_index) }
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
            new_component_data
                // Chain an iterator of component pointers from the old archetype
                .chain(old_archetype.components.iter().filter_map(|info| {
                    if self.components.contains(info.id) {
                        Some(
                            old_archetype
                                .pointer_to_entity_component(entity_meta.table_index, info.id)
                                .map(|ptr| (info.id, ptr.as_ptr() as *const u8))
                                .expect("entity not found in old archetype"),
                        )
                    } else {
                        None
                    }
                })),
        );
        self.entities.push(entity_id);
    }

    pub fn get_columns(&self, ids: &[ComponentId]) -> Vec<Option<Rc<Column>>> {
        ids.iter()
            .map(|id| self.table.columns.get(id).map(Rc::clone))
            .collect()
    }

    pub fn get_column(&self, id: ComponentId) -> Option<Rc<Column>> {
        self.table.columns.get(&id).map(Rc::clone)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub usize);

impl ArchetypeId {
    pub const EMPTY: Self = ArchetypeId(0);
}

#[derive(Debug)]
pub struct Column {
    pub(crate) data: UnsafeCell<AnyVec>,
    pub(crate) change_detection: &'static UnsafeCell<ChangeDetectionContext>,
}

impl Column {
    pub fn new(data: UnsafeCell<AnyVec>) -> Self {
        Self {
            data,
            change_detection: Box::leak(Box::new(UnsafeCell::default())),
        }
    }

    pub unsafe fn push_from_ptr(&self, ptr: *const u8) {
        (&mut *self.data.get()).push_from_ptr(ptr);
        (*self.change_detection.get())
            .changed_ticks
            .push(Tick::new(0));
    }

    pub unsafe fn remove(&self, index: usize) {
        (*self.data.get()).swap_remove(index);
        (*self.change_detection.get())
            .changed_ticks
            .swap_remove(index);
    }

    pub fn get_changed_tick(&self, index: usize) -> Tick {
        unsafe { (*self.change_detection.get()).changed_ticks[index] }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Edges {
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

    pub fn init_archetype(&mut self, archetype_id: ArchetypeId) {
        self.insert_edges.insert(archetype_id, HashMap::new());
        self.remove_edges.insert(archetype_id, HashMap::new());
    }
}
