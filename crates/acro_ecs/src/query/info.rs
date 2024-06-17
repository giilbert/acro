use std::ptr::NonNull;

use crate::{
    archetype::ArchetypeId,
    registry::{ComponentGroup, ComponentInfo},
    world::World,
};

use super::utils::QueryInfoUtils;

#[derive(Debug)]
pub struct QueryInfo {
    pub(super) archetypes_generation: usize,
    pub(super) archetypes: Vec<ArchetypeId>,
    pub(super) components: Vec<QueryComponentInfo>,
}

impl QueryInfo {
    pub fn recompute_archetypes(&mut self, world: &mut World) {
        self.archetypes = find_archetypes(world, &self.components);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum QueryComponentInfo {
    Borrowed(ComponentInfo),
    BorrowedMut(ComponentInfo),
    OptionBorrow(ComponentInfo),
    OptionBorrowMut(ComponentInfo),
}

impl QueryComponentInfo {
    pub fn component_info(&self) -> &ComponentInfo {
        match self {
            QueryComponentInfo::Borrowed(info) => info,
            QueryComponentInfo::BorrowedMut(info) => info,
            QueryComponentInfo::OptionBorrow(info) => info,
            QueryComponentInfo::OptionBorrowMut(info) => info,
        }
    }
}

pub trait ToQueryInfo {
    fn to_query_info(world: &mut World) -> QueryInfo;
    unsafe fn from_parts(components: impl Iterator<Item = NonNull<u8>>) -> Self;
}

fn get_full_component_info<T: QueryInfoUtils>(world: &mut World) -> QueryComponentInfo {
    if T::is_borrowed_mut() {
        QueryComponentInfo::BorrowedMut(world.get_component_info::<T>().clone())
    } else {
        QueryComponentInfo::Borrowed(world.get_component_info::<T>().clone())
    }
}

fn find_archetypes(world: &mut World, components: &[QueryComponentInfo]) -> Vec<ArchetypeId> {
    let component_group = ComponentGroup::new(
        components
            .iter()
            .map(|c| c.component_info().clone())
            .collect(),
    );

    world.archetypes.get_archetypes_with(&component_group)
}

macro_rules! impl_to_query_info {
    ($($members:ident),+) => {
        impl<$($members: QueryInfoUtils),*> ToQueryInfo for ($($members,)*) {
            fn to_query_info(world: &mut World) -> QueryInfo {
                let components = vec![$(get_full_component_info::<$members>(world),)*];
                QueryInfo {
                    archetypes_generation: world.archetypes.generation,
                    archetypes: find_archetypes(world, &components),
                    components,
                }
            }

            #[inline]
            unsafe fn from_parts(mut components: impl Iterator<Item = NonNull<u8>>) -> Self {
                (
                    $(
                        unsafe {
                            let component =
                                components.next().expect("unable to find component reference");
                            // component is a pointer to the data
                            std::mem::transmute_copy::<_, $members>(&component.as_ptr())
                        },
                    )*
                )
            }
        }
    };
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
