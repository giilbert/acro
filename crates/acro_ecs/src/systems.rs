use std::{any::Any, rc::Rc};

use crate::{
    pointer::change_detection::Tick,
    query::{Query, QueryFilter, QueryInfo, ToQueryInfo},
    world::World,
};

#[derive(Debug, Clone)]
pub struct SystemRunContext<'w> {
    pub world: &'w World,
    pub tick: Tick,
    pub last_run_tick: Tick,
}

impl SystemRunContext<'_> {
    pub fn new(world: &World, tick: Tick) -> SystemRunContext {
        SystemRunContext {
            world,
            tick,
            last_run_tick: Tick::new(0),
        }
    }

    pub fn ignore_changes(world: &World) -> SystemRunContext {
        SystemRunContext::new(world, Tick::new(0))
    }
}

pub trait IntoSystemRunContext<'w> {
    fn into_system_run_context(self) -> SystemRunContext<'w>;
}

impl<'w> IntoSystemRunContext<'w> for &'w World {
    fn into_system_run_context(self) -> SystemRunContext<'w> {
        SystemRunContext::new(self, Tick::new(0))
    }
}

impl<'w> IntoSystemRunContext<'w> for SystemRunContext<'w> {
    fn into_system_run_context(self) -> SystemRunContext<'w> {
        self
    }
}

impl<'w> IntoSystemRunContext<'w> for &'w SystemRunContext<'w> {
    fn into_system_run_context(self) -> SystemRunContext<'w> {
        self.clone()
    }
}

pub trait SystemParam<'w> {
    type Init: Any;

    fn init(world: &World) -> Self::Init;
    fn create(prepared: &mut Self::Init) -> Self;
}

impl<'w, TData, TFilters> SystemParam<'w> for Query<TData, TFilters>
where
    TData: for<'a> ToQueryInfo<'a>,
    TFilters: for<'a> QueryFilter<'a>,
{
    type Init = Rc<QueryInfo>;

    fn init(world: &World) -> Self::Init {
        Rc::new(TData::to_query_info::<TFilters>(world))
    }

    fn create(prepared: &mut Self::Init) -> Self {
        Query {
            info: Rc::clone(prepared),
            _phantom: Default::default(),
        }
    }
}

pub trait IntoSystem<P> {
    fn init(world: &World) -> Box<dyn Any>;
    fn into_system(self) -> Box<dyn FnOnce(&SystemRunContext, &mut dyn Any) -> ()>;
}

impl<F, P1> IntoSystem<P1> for F
where
    F: Fn(&SystemRunContext, P1) + 'static,
    P1: for<'a> SystemParam<'a>,
{
    fn init(world: &World) -> Box<dyn Any> {
        Box::new(P1::init(world))
    }

    fn into_system(self) -> Box<dyn FnOnce(&SystemRunContext, &mut dyn Any) -> ()> {
        Box::new(move |context, parameters| {
            let parameters = parameters.downcast_mut::<P1::Init>().unwrap();
            self(&context, P1::create(parameters));
        })
    }
}

macro_rules! impl_into_system {
    ($($members:ident),+) => {
        impl<
            F: Fn(&SystemRunContext, $($members),+) + 'static,
            $($members: for<'a> SystemParam<'a>),*
        > IntoSystem<($($members),+)> for F {
            fn init(world: &World) -> Box<dyn Any> {
                Box::new(($($members::init(world),)*))
            }

            #[allow(non_snake_case)]
            fn into_system(self) -> Box<dyn FnOnce(&SystemRunContext, &mut dyn Any) -> ()> {
                Box::new(move |context, parameters| {
                    let ($($members,)*) = parameters.downcast_mut::<($($members::Init,)*)>().unwrap();
                    self(context, $($members::create($members),)*);
                })
            }
        }
    }
}

impl_into_system!(P1, P2);
impl_into_system!(P1, P2, P3);
impl_into_system!(P1, P2, P3, P4);
impl_into_system!(P1, P2, P3, P4, P5);
impl_into_system!(P1, P2, P3, P4, P5, P6);
impl_into_system!(P1, P2, P3, P4, P5, P6, P7);
impl_into_system!(P1, P2, P3, P4, P5, P6, P7, P8);
