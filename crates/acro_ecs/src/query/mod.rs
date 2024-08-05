mod filters;
mod info;
mod iter;
mod transform;
mod utils;

pub use filters::{Changed, Or, QueryFilter, With, Without};
pub use info::{QueryInfo, ToQueryInfo};
use tracing::info;

use std::{fmt::Debug, marker::PhantomData, rc::Rc};

use crate::{
    entity::EntityId,
    systems::{IntoSystemRunContext, SystemRunContext},
    world::World,
};

use self::iter::QueryIter;

#[derive(Debug)]
pub struct Query<T: ToQueryInfo, F: QueryFilter = ()> {
    pub(super) info: Rc<QueryInfo>,
    pub(super) _phantom: PhantomData<(T, F)>,
}

impl<T, F> Query<T, F>
where
    T: ToQueryInfo,
    F: QueryFilter + 'static,
{
    pub fn new<TData, TFilters>(world: &World) -> Query<TData, TFilters>
    where
        TData: ToQueryInfo,
        TFilters: QueryFilter,
    {
        let info = TData::to_query_info::<TFilters>(world);

        Query {
            info: Rc::new(info),
            _phantom: PhantomData,
        }
    }

    pub fn check_archetypes(&self, world: &World) {
        if *self.info.archetypes_generation.borrow() < world.archetypes.generation {
            self.info.recompute_archetypes::<F>(world);
            *self.info.archetypes_generation.borrow_mut() = world.archetypes.generation;
        }
    }

    pub fn get_single<'w>(
        &self,
        ctx: impl IntoSystemRunContext<'w>,
    ) -> Option<<T as ToQueryInfo>::Output> {
        self.over(ctx).next()
    }

    pub fn single<'w>(&self, ctx: impl IntoSystemRunContext<'w>) -> <T as ToQueryInfo>::Output {
        self.get_single(ctx).expect("query returned no results")
    }

    pub fn over<'w, 'q>(&'q self, ctx: impl IntoSystemRunContext<'w>) -> QueryIter<'w, 'q, T, F> {
        let ctx = ctx.into_system_run_context();
        self.check_archetypes(ctx.world);
        QueryIter::new(ctx, self)
    }

    pub fn get<'w>(
        &self,
        ctx: impl IntoSystemRunContext<'w>,
        entity_id: EntityId,
    ) -> Option<<T as ToQueryInfo>::Output> {
        let ctx = ctx.into_system_run_context();
        let entity_meta = ctx.world.entity_meta(entity_id);
        let archetype_id = entity_meta.archetype_id;
        ctx.world
            .archetypes
            .get_archetype(archetype_id)
            .and_then(|archetype| {
                let archetype = archetype.borrow();
                let column = archetype.get_columns(&self.info.component_ids);

                let ret = Some(unsafe {
                    T::from_parts(
                        &ctx,
                        &archetype,
                        entity_meta.table_index,
                        self.info
                            .component_ids
                            .iter()
                            .zip(column)
                            .map(|(&component_id, c)| {
                                (
                                    component_id,
                                    c.as_ref()
                                        .map(|column| {
                                            (&*column.data.get()).get_ptr(entity_meta.table_index)
                                        })
                                        .flatten(),
                                )
                            }),
                    )
                });

                ret
            })
    }
}
#[cfg(test)]
mod tests {
    use assert_unordered::assert_eq_unordered;

    use crate::{entity::EntityId, query::info::QueryComponentInfo, world::World};

    #[test]
    fn query() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<bool>();
        world.init_component::<String>();

        let entity1 = world.spawn((42u32, "hello".to_string()));
        let entity2 = world.spawn((12u32, "bye".to_string(), true));
        let _entity3 = world.spawn((42u32, false));

        let query1 = world.query::<(&u32, &mut String), ()>();
        assert_eq!(
            query1.info.components,
            vec![
                QueryComponentInfo::Borrowed(world.get_component_info::<u32>().clone()),
                QueryComponentInfo::BorrowedMut(world.get_component_info::<String>().clone())
            ]
        );
        assert_eq_unordered!(
            &*query1.info.archetypes.borrow(),
            &vec![
                world.entity_meta(entity2).archetype_id,
                world.entity_meta(entity1).archetype_id
            ]
        );

        let data1 = query1
            .over(&world)
            .map(|(x, y)| (x, y.clone()))
            .collect::<Vec<_>>();
        assert_eq_unordered!(
            data1,
            vec![(&42u32, "hello".to_string()), (&12u32, "bye".to_string())]
        );

        let query2 = world.query::<(&u32, &bool), ()>();
        let data2 = query2.over(&world).collect::<Vec<_>>();
        assert_eq_unordered!(data2, vec![(&12u32, &true), (&42u32, &false)]);
    }

    #[test]
    fn query_after_changing_archetypes() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<bool>();

        let entity1 = world.spawn((42u32,));
        let entity2 = world.spawn((12u32,));

        let query1 = world.query::<&u32, ()>();
        let data1 = query1.over(&world).collect::<Vec<_>>();
        assert_eq_unordered!(data1, vec![&42u32, &12u32]);

        world.insert(entity1, true);

        let data2 = query1.over(&world).collect::<Vec<_>>();
        assert_eq_unordered!(data2, vec![&42u32, &12u32]);

        world.insert(entity2, false);

        let data3 = query1.over(&world).collect::<Vec<_>>();
        assert_eq_unordered!(data3, vec![&42u32, &12u32]);
    }

    #[test]
    fn query_with_entity_id() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<bool>();

        let entity1 = world.spawn((42u32,));
        let entity2 = world.spawn((12u32,));

        let query1 = world.query::<(EntityId, &u32), ()>();
        let data1 = query1.over(&world).collect::<Vec<_>>();
        assert_eq_unordered!(data1, vec![(entity1, &42u32), (entity2, &12u32)]);
    }

    #[test]
    fn query_with_options() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<u8>();
        world.init_component::<bool>();

        let entity1 = world.spawn((42u32,));
        let entity2 = world.spawn((12u32, true));
        let entity3 = world.spawn((2u32, 42u8, false));
        let entity4 = world.spawn((2u32, 42u8));

        let query1 = world.query::<(EntityId, Option<&bool>), ()>();
        let data1 = query1.over(&world).collect::<Vec<_>>();

        assert_eq_unordered!(
            data1,
            vec![
                (entity1, None),
                (entity2, Some(&true)),
                (entity3, Some(&false)),
                (entity4, None)
            ]
        );
    }

    #[test]
    fn query_no_archetypes() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<bool>();

        let _entity = world.spawn_empty();

        let query1 = world.query::<&u32, ()>();
        let data1 = query1.over(&world).collect::<Vec<&u32>>();
        assert!(data1.is_empty());
    }
}
