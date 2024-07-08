use std::{collections::HashSet, option, process::Output, ptr::NonNull};

use itertools::Itertools;

use crate::{
    archetype::{Archetype, ArchetypeId},
    registry::{ComponentGroup, ComponentId, ComponentInfo},
    world::World,
};

use super::{
    transform::QueryTransform,
    utils::{QueryBorrowType, QueryFetchType, QueryInfoUtils},
};

#[derive(Debug)]
pub struct QueryInfo {
    pub(super) archetypes_generation: usize,
    pub(super) archetypes: Vec<ArchetypeId>,
    pub(super) components: Vec<QueryComponentInfo>,
}

impl QueryInfo {
    pub fn recompute_archetypes(&mut self, world: &World) {
        self.archetypes = find_archetypes(world, &self.components);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum QueryComponentInfo {
    EntityId,
    Borrowed(ComponentInfo),
    BorrowedMut(ComponentInfo),
    OptionBorrow(ComponentInfo),
    OptionBorrowMut(ComponentInfo),
}

impl QueryComponentInfo {
    pub fn is_component(&self) -> bool {
        match self {
            QueryComponentInfo::EntityId => false,
            _ => true,
        }
    }

    pub fn is_required(&self) -> bool {
        match self {
            QueryComponentInfo::Borrowed(_) => true,
            QueryComponentInfo::BorrowedMut(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn component_info(&self) -> &ComponentInfo {
        match self {
            QueryComponentInfo::Borrowed(info) => info,
            QueryComponentInfo::BorrowedMut(info) => info,
            QueryComponentInfo::OptionBorrow(info) => info,
            QueryComponentInfo::OptionBorrowMut(info) => info,
            QueryComponentInfo::EntityId => panic!("EntityId has no component info"),
        }
    }
}

pub trait ToQueryInfo<'w> {
    type Output;

    fn to_query_info(world: &mut World) -> QueryInfo;
    unsafe fn from_parts(
        world: &'w World,
        current_archetype: &Archetype,
        entity_index: usize,
        components: impl Iterator<Item = (ComponentId, Option<NonNull<u8>>)>,
    ) -> Self::Output;
}

fn get_full_component_info<T: QueryInfoUtils>(world: &mut World) -> QueryComponentInfo {
    match (T::BORROW, T::FETCH) {
        (_, QueryFetchType::EntityId) => return QueryComponentInfo::EntityId,
        _ => (),
    }

    let component_info = world
        .get_component_info_id(<T as QueryInfoUtils>::type_id())
        .clone();

    match (T::BORROW, T::FETCH) {
        (QueryBorrowType::Borrow, QueryFetchType::Component) => {
            QueryComponentInfo::Borrowed(component_info)
        }
        (QueryBorrowType::BorrowMut, QueryFetchType::Component) => {
            QueryComponentInfo::BorrowedMut(component_info)
        }
        (QueryBorrowType::OptionBorrow, QueryFetchType::Component) => {
            QueryComponentInfo::OptionBorrow(component_info)
        }
        (QueryBorrowType::OptionBorrowMut, QueryFetchType::Component) => {
            QueryComponentInfo::OptionBorrowMut(component_info)
        }
        _ => unimplemented!(
            "unsupported component query type: {:?} {:?}",
            T::BORROW,
            T::FETCH
        ),
    }
}

fn find_archetypes(world: &World, components: &[QueryComponentInfo]) -> Vec<ArchetypeId> {
    let required_components = components
        .iter()
        .filter(|c| c.is_component() && c.is_required())
        .collect_vec();
    let optional_components = components
        .iter()
        .filter(|c| c.is_component() && !c.is_required())
        .collect_vec();

    let set: HashSet<ArchetypeId> = HashSet::from_iter(
        optional_components
            .iter()
            .powerset()
            .map(|c| {
                let component_group = ComponentGroup::new(
                    c.into_iter()
                        .chain(required_components.iter())
                        .map(|c| c.component_info().clone())
                        .collect(),
                );

                world.archetypes.get_archetypes_with(&component_group)
            })
            .flatten()
            .collect_vec(),
    );

    set.into_iter().collect()
}

macro_rules! impl_to_query_info {
    ($($members:ident),+) => {
        impl<
            'w,
            $($members: QueryInfoUtils + QueryTransform<'w, InputOrCreate = $members>),*
        > ToQueryInfo<'w> for ($($members,)*) {
            type Output = ($(<$members as QueryTransform<'w>>::Output,)*);

            fn to_query_info(world: &mut World) -> QueryInfo {
                let components = vec![$(get_full_component_info::<$members>(world),)*];
                QueryInfo {
                    archetypes_generation: world.archetypes.generation,
                    archetypes: find_archetypes(world, &components),
                    components,
                }
            }

            #[inline]
            unsafe fn from_parts(
                world: &'w World,
                current_archetype: &Archetype,
                entity_index: usize,
                mut components: impl Iterator<Item = (ComponentId, Option<NonNull<u8>>)>,
            ) -> Self::Output {
                (
                    $(
                        if <$members as QueryTransform>::IS_CREATE {
                            <$members as QueryTransform>::create(
                                world,
                                current_archetype,
                                entity_index,
                            )
                        } else {
                            let (component_id, component) = components
                                .next()
                                .expect("unable to find componet reference");

                            <$members as QueryTransform>::transform_component(
                                world,
                                current_archetype,
                                entity_index,
                                component_id,
                                std::mem::transmute_copy::<_, $members>(&component),
                            )
                        },
                    )*
                )
            }
        }
    }
}

impl_to_query_info!(T1);
impl_to_query_info!(T1, T2);
impl_to_query_info!(T1, T2, T3);
impl_to_query_info!(T1, T2, T3, T4);
impl_to_query_info!(T1, T2, T3, T4, T5);
impl_to_query_info!(T1, T2, T3, T4, T5, T6);
impl_to_query_info!(T1, T2, T3, T4, T5, T6, T7);
impl_to_query_info!(T1, T2, T3, T4, T5, T6, T7, T8);

pub trait ToFilterInfo {}

impl ToFilterInfo for () {}
