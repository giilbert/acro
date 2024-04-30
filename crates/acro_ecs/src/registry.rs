use std::{
    alloc::Layout,
    any::TypeId,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use fnv::FnvHashMap;

#[derive(Debug, Default)]
pub struct ComponentRegistry {
    current_id: usize,
    native_components: FnvHashMap<TypeId, ComponentId>,
    components: FnvHashMap<ComponentId, ComponentInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub usize);

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub id: ComponentId,
    pub component_type: ComponentType,
}

#[derive(Debug, Clone)]
pub enum ComponentType {
    Native {
        name: String,
        layout: Layout,
        type_id: TypeId,
    },
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            current_id: 0,
            native_components: HashMap::default(),
            components: HashMap::default(),
        }
    }

    fn next_id(&mut self) -> ComponentId {
        let id = ComponentId(self.current_id);
        self.current_id += 1;
        id
    }

    pub fn get<T: 'static>(&self) -> Option<&ComponentInfo> {
        self.components
            .get(self.native_components.get(&TypeId::of::<T>())?)
    }

    pub fn init<T: 'static>(&mut self) -> &ComponentInfo {
        let id = self.next_id();

        self.components.insert(
            id,
            ComponentInfo {
                id,
                component_type: ComponentType::Native {
                    name: std::any::type_name::<T>().to_string(),
                    layout: Layout::new::<T>(),
                    type_id: TypeId::of::<T>(),
                },
            },
        );
        self.native_components.insert(TypeId::of::<T>(), id);

        self.components
            .get(&id)
            .expect("unable to find component info")
    }

    pub fn get_info(&self, component_id: ComponentId) -> Option<&ComponentInfo> {
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
    ids: Vec<usize>,
}

impl ComponentGroup {
    pub fn new(mut data: Vec<ComponentInfo>) -> Self {
        data.sort_by(|left, right| left.id.0.cmp(&right.id.0));
        let ids = data.iter().map(|info| info.id.0).collect();
        Self { data, ids }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ComponentInfo> {
        self.data.iter()
    }

    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.ids.contains(&component_id.0)
    }

    pub fn extend(&self, component: ComponentInfo) -> Self {
        let mut data = self.data.clone();
        data.push(component);
        Self::new(data)
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
