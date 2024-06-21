mod info;
mod iter;
mod transform;
mod utils;

pub use info::{ToFilterInfo, ToQueryInfo};

use std::marker::PhantomData;

use crate::{registry::ComponentId, world::World};

use self::{info::QueryInfo, iter::QueryIter};

pub struct Query<T: for<'w> ToQueryInfo<'w>, F: ToFilterInfo> {
    pub(super) info: QueryInfo,
    pub(super) component_ids: Vec<ComponentId>,
    _phantom: PhantomData<(T, F)>,
}

impl<T, F> Query<T, F>
where
    T: for<'w> ToQueryInfo<'w>,
    F: ToFilterInfo,
{
    pub fn new<TData, TFilters>(world: &mut World) -> Query<TData, TFilters>
    where
        TData: for<'w> ToQueryInfo<'w>,
        TFilters: ToFilterInfo,
    {
        let info = TData::to_query_info(world);
        let component_ids = info
            .components
            .iter()
            .map(|c| c.component_info().id)
            .collect::<Vec<ComponentId>>();

        Query {
            info,
            component_ids,
            _phantom: PhantomData,
        }
    }

    pub fn over<'w, 'q>(&'q mut self, world: &'w World) -> QueryIter<'w, 'q, T, F> {
        if self.info.archetypes_generation != world.archetypes.generation {
            self.info.recompute_archetypes(world);
        }

        QueryIter::new(world, self)
    }
}

#[cfg(test)]
mod tests {
    use assert_unordered::assert_eq_unordered;

    use crate::{archetype::ArchetypeId, query::info::QueryComponentInfo, world::World};

    #[test]
    fn query() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<bool>();
        world.init_component::<String>();

        let entity1 = world.spawn();
        world.insert(entity1, 42u32);
        world.insert(entity1, "hello".to_string());

        let entity2 = world.spawn();
        world.insert(entity2, 12u32);
        world.insert(entity2, "bye".to_string());
        world.insert(entity2, true);

        let entity3 = world.spawn();
        world.insert(entity3, 42u32);
        world.insert(entity3, false);

        let mut query1 = world.query::<(&u32, &mut String), ()>();
        assert_eq!(
            query1.info.components,
            vec![
                QueryComponentInfo::Borrowed(world.get_component_info::<u32>().clone()),
                QueryComponentInfo::BorrowedMut(world.get_component_info::<String>().clone())
            ]
        );
        assert_eq_unordered!(
            &query1.info.archetypes,
            &vec![
                world.entity_meta(entity2).archetype_id,
                world.entity_meta(entity1).archetype_id
            ]
        );

        let data1 = query1
            .over(&mut world)
            .map(|(x, y)| (x, y.clone()))
            .collect::<Vec<_>>();
        assert_eq_unordered!(
            data1,
            vec![(&42u32, "hello".to_string()), (&12u32, "bye".to_string())]
        );

        let mut query2 = world.query::<(&u32, &bool), ()>();
        let data2 = query2.over(&mut world).collect::<Vec<_>>();
        assert_eq_unordered!(data2, vec![(&12u32, &true), (&42u32, &false)]);
    }

    #[test]
    fn query_after_changing_archetypes() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<bool>();

        let entity1 = world.spawn();
        world.insert(entity1, 42u32);

        let entity2 = world.spawn();
        world.insert(entity2, 12u32);

        let mut query1 = world.query::<(&u32,), ()>();
        let data1 = query1.over(&mut world).collect::<Vec<_>>();
        assert_eq_unordered!(data1, vec![(&42u32,), (&12u32,)]);

        world.insert(entity1, true);

        let data2 = query1.over(&mut world).collect::<Vec<_>>();
        assert_eq_unordered!(data2, vec![(&42u32,), (&12u32,)]);

        world.insert(entity2, false);

        let data3 = query1.over(&mut world).collect::<Vec<_>>();
        assert_eq_unordered!(data3, vec![(&42u32,), (&12u32,)]);
    }
}
