mod info;
mod iter;
mod utils;

pub use info::{ToFilterInfo, ToQueryInfo};

use std::{any::Any, marker::PhantomData};

use crate::{registry::ComponentInfo, world::World};

use self::{info::QueryInfo, iter::ExclusiveQueryIter};

pub struct Query<T: ToQueryInfo, F: ToFilterInfo> {
    pub(super) info: QueryInfo,
    _phantom: PhantomData<(T, F)>,
}

impl<T, F> Query<T, F>
where
    T: ToQueryInfo,
    F: ToFilterInfo,
{
    pub fn new<TData, TFilters>(world: &mut World) -> Query<TData, TFilters>
    where
        TData: ToQueryInfo,
        TFilters: ToFilterInfo,
    {
        Query {
            info: TData::to_query_info(world),
            _phantom: PhantomData,
        }
    }

    pub fn exclusive<'w, 'q>(&'q self, world: &'w mut World) -> ExclusiveQueryIter<'w, 'q, T, F> {
        ExclusiveQueryIter::new(world, self)
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

        let entity1 = world.spawn();
        world.insert(entity1, 42u32);
        world.insert(entity1, "hello".to_string());

        let entity2 = world.spawn();
        world.insert(entity2, 12u32);
        world.insert(entity2, "bye".to_string());

        let query = world.query::<(&u32, &mut String), ()>();
        assert_eq!(
            query.info.components,
            vec![
                QueryComponentInfo::Borrowed(world.get_component_info::<u32>().clone()),
                QueryComponentInfo::BorrowedMut(world.get_component_info::<String>().clone())
            ]
        );
        assert_eq!(
            query.info.parent_archetype_id,
            world.entity_meta(entity1).archetype_id
        );

        let data = query.exclusive(&mut world).collect::<Vec<_>>();
        assert_eq!(
            data,
            vec![
                (&42u32, &mut "hello".to_string()),
                (&12u32, &mut "bye".to_string())
            ]
        );
    }
}
