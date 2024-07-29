mod filters;
mod info;
mod iter;
mod transform;
mod utils;

pub use filters::{Changed, Or, QueryFilter, With, Without};
pub use info::{QueryInfo, ToQueryInfo};

use std::{any::Any, fmt::Debug, marker::PhantomData, rc::Rc};

use crate::{
    archetype,
    entity::{self, EntityId},
    registry::ComponentId,
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
    F: QueryFilter,
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

    pub fn over<'w, 'q>(&'q self, ctx: impl IntoSystemRunContext<'w>) -> QueryIter<'w, 'q, T, F> {
        let ctx = ctx.into_system_run_context();

        if self.info.archetypes_generation != ctx.world.archetypes.generation {
            self.info.recompute_archetypes::<F>(ctx.world);
        }

        QueryIter::new(ctx, self)
    }

    pub fn get(
        &self,
        ctx: &SystemRunContext,
        entity_id: EntityId,
    ) -> Option<<T as ToQueryInfo>::Output> {
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

    use crate::{
        archetype::ArchetypeId, entity::EntityId, pointer::change_detection::Tick,
        query::info::QueryComponentInfo, systems::SystemRunContext, world::World,
    };

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

        let mut query2 = world.query::<(&u32, &bool), ()>();
        let data2 = query2.over(&world).collect::<Vec<_>>();
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

        let mut query1 = world.query::<&u32, ()>();
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

        let entity1 = world.spawn();
        world.insert(entity1, 42u32);

        let entity2 = world.spawn();
        world.insert(entity2, 12u32);

        let mut query1 = world.query::<(EntityId, &u32), ()>();
        let data1 = query1.over(&world).collect::<Vec<_>>();
        assert_eq_unordered!(data1, vec![(entity1, &42u32), (entity2, &12u32)]);
    }

    #[test]
    fn query_with_options() {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<u8>();
        world.init_component::<bool>();

        let entity1 = world.spawn();
        world.insert(entity1, 42u32);

        let entity2 = world.spawn();
        world.insert(entity2, 12u32);
        world.insert(entity2, true);

        let entity3 = world.spawn();
        world.insert(entity3, 2u32);
        world.insert(entity3, 42u8);
        world.insert(entity3, false);

        let entity4 = world.spawn();
        world.insert(entity4, 2u32);
        world.insert(entity4, 42u8);

        let mut query1 = world.query::<(EntityId, Option<&bool>), ()>();
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

        let _entity = world.spawn();

        let query1 = world.query::<&u32, ()>();
        let data1 = query1.over(&world).collect::<Vec<&u32>>();
        assert!(data1.is_empty());
    }
}
