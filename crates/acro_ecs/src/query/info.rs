use crate::{
    archetype::ArchetypeId,
    registry::{ComponentGroup, ComponentInfo},
    world::World,
};

use super::utils::QueryInfoUtils;

#[derive(Debug)]
pub struct QueryInfo {
    parent_archetype_id: ArchetypeId,
    pub(crate) components: Vec<QueryComponentInfo>,
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
}

fn get_full_component_info<T: QueryInfoUtils>(world: &mut World) -> QueryComponentInfo {
    if T::is_borrowed_mut() {
        QueryComponentInfo::BorrowedMut(world.get_component_info::<T>().clone())
    } else {
        QueryComponentInfo::Borrowed(world.get_component_info::<T>().clone())
    }
}

fn find_parent_archetype(world: &World, components: &[QueryComponentInfo]) -> ArchetypeId {
    let component_group = ComponentGroup::new(
        components
            .iter()
            .map(|c| c.component_info().clone())
            .collect(),
    );

    todo!();
}

macro_rules! impl_to_query_info {
    ($($members:ident),+) => {
        impl<$($members: QueryInfoUtils),*> ToQueryInfo for ($($members,)*) {
            fn to_query_info(world: &mut World) -> QueryInfo {
                let components =vec![$(get_full_component_info::<$members>(world),)*];
                QueryInfo {
                    parent_archetype_id: find_parent_archetype(world, &components),
                    components,
                }
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
