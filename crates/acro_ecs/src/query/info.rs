use crate::{registry::ComponentInfo, world::World};

use super::utils::QueryInfoUtils;

pub struct QueryInfo {
    pub(crate) components: Vec<QueryComponentInfo>,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum QueryComponentInfo {
    BorrowedComponent(ComponentInfo),
    BorrowedMutComponent(ComponentInfo),
}

pub trait ToQueryInfo {
    fn to_query_info(world: &mut World) -> QueryInfo;
}

fn get_full_component_info<T: QueryInfoUtils>(world: &mut World) -> QueryComponentInfo {
    if T::is_borrowed_mut() {
        QueryComponentInfo::BorrowedMutComponent(world.get_component_info::<T>().clone())
    } else {
        QueryComponentInfo::BorrowedComponent(world.get_component_info::<T>().clone())
    }
}

macro_rules! impl_to_query_info {
    ($($members:ident),+) => {
        impl<$($members: QueryInfoUtils),*> ToQueryInfo for ($($members,)*) {
            fn to_query_info(world: &mut World) -> QueryInfo {
                QueryInfo {
                    components: vec![
                        $(get_full_component_info::<$members>(world),)*
                    ],
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
