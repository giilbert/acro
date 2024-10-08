use std::{
    alloc::Layout,
    any::TypeId,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

use crate::{storage::anyvec::Dropper, Name};

#[derive(Debug, Default)]
pub struct ComponentRegistry {
    current_id: u32,
    native_components: FnvHashMap<TypeId, ComponentId>,
    components: FnvHashMap<ComponentId, ComponentInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentInfo {
    pub id: ComponentId,
    pub component_type: ComponentType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Native {
        name: String,
        layout: Layout,
        type_id: TypeId,
        dropper: Dropper,
    },
}

impl ComponentRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            current_id: 0,
            native_components: HashMap::default(),
            components: HashMap::default(),
        };

        registry.init_rust_type::<Name>();

        registry
    }

    fn next_id(&mut self) -> ComponentId {
        let id = ComponentId(self.current_id);
        self.current_id += 1;
        id
    }

    pub fn get<T: 'static>(&self) -> Option<&ComponentInfo> {
        self.get_by_id(TypeId::of::<T>())
    }

    pub fn get_by_id(&self, id: TypeId) -> Option<&ComponentInfo> {
        self.native_components
            .get(&id)
            .and_then(|id| self.components.get(id))
    }

    pub fn init_rust_type<T: 'static>(&mut self) -> &ComponentInfo {
        let id = self.next_id();

        self.components.insert(
            id,
            ComponentInfo {
                id,
                component_type: ComponentType::Native {
                    name: std::any::type_name::<T>().to_string(),
                    layout: Layout::new::<T>(),
                    type_id: TypeId::of::<T>(),
                    dropper: Some(|ptr| unsafe {
                        std::ptr::drop_in_place(ptr.as_ptr() as *mut T);
                    }),
                },
            },
        );

        self.native_components.insert(TypeId::of::<T>(), id);

        self.components
            .get(&id)
            .expect("unable to find component info")
    }

    /// Get the component info for a given component id, panicking if the component is not found
    #[inline]
    pub fn get_info(&self, component_id: ComponentId) -> &ComponentInfo {
        self.try_get_info(component_id)
            .expect("component not found")
    }

    #[inline]
    pub fn try_get_info(&self, component_id: ComponentId) -> Option<&ComponentInfo> {
        self.components.get(&component_id)
    }

    pub fn create_group(&self, component_ids: HashSet<ComponentId>) -> ComponentGroup {
        let data = self
            .components
            .iter()
            .filter(|(id, _)| component_ids.contains(id))
            .map(|(_, info)| info.clone())
            .collect();

        ComponentGroup::new(data)
    }
}
#[derive(Debug, Clone)]
pub struct ComponentGroup {
    data: Vec<ComponentInfo>,
    ids: Vec<ComponentId>,
}

impl ComponentGroup {
    pub fn new(mut data: Vec<ComponentInfo>) -> Self {
        data.sort_by(|left, right| left.id.0.cmp(&right.id.0));
        let ids = data.iter().map(|info| info.id).collect();
        Self { data, ids }
    }

    pub fn new_unsorted(data: Vec<ComponentInfo>) -> Self {
        let ids = data.iter().map(|info| info.id).collect();
        Self { data, ids }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ComponentInfo> {
        self.data.iter()
    }

    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.ids.iter().any(|id| *id == component_id)
    }

    pub fn extend(&self, component: ComponentInfo) -> Self {
        let mut data = self.data.clone();
        data.push(component);
        Self::new(data)
    }

    pub fn remove(&self, component: ComponentInfo) -> Self {
        let data = self
            .data
            .clone()
            .into_iter()
            .filter(|info| info.id != component.id)
            .collect();
        Self::new(data)
    }

    pub fn is_subset_of(&self, other: &ComponentGroup) -> bool {
        self.ids.iter().all(|id| other.contains(*id))
    }
}

impl Hash for ComponentGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ids.hash(state);
    }
}

impl PartialEq for ComponentGroup {
    fn eq(&self, other: &Self) -> bool {
        self.ids == other.ids
    }
}

impl Eq for ComponentGroup {}
