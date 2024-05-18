mod info;
mod utils;

pub use info::ToQueryInfo;

use std::{any::Any, marker::PhantomData};

use crate::{registry::ComponentInfo, world::World};

use self::info::QueryInfo;

pub struct Query<T: ToQueryInfo, F> {
    info: QueryInfo,
    _phantom: PhantomData<(T, F)>,
}

impl<T, F> Query<T, F>
where
    T: ToQueryInfo,
{
    pub fn new<TData, TFilters>(world: &mut World) -> Query<TData, TFilters>
    where
        TData: ToQueryInfo,
    {
        Query {
            info: TData::to_query_info(world),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{query::info::QueryComponentInfo, world::World};

    #[test]
    fn query() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<String>();

        let entity = world.spawn();
        world.insert(entity, 42u32);
        world.insert(entity, "hello".to_string());

        let query = world.query::<(&u32, &mut String), ()>();
        assert_eq!(
            query.info.components,
            vec![
                QueryComponentInfo::Borrowed(world.get_component_info::<u32>().clone()),
                QueryComponentInfo::BorrowedMut(world.get_component_info::<String>().clone())
            ]
        );
    }
}
