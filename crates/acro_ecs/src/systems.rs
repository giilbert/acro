use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use crate::{
    pointer::change_detection::Tick,
    query::{Query, QueryFilter, QueryInfo, ToQueryInfo},
    schedule::SystemSchedulingRequirement,
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

pub type SystemFn = Box<dyn Fn(&mut World, Tick, &mut dyn Any)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemId {
    Native(TypeId),
    Faux(usize),
}

pub struct SystemData {
    pub id: SystemId,
    pub name: String,
    pub run: SystemFn,
    pub last_run_tick: Tick,
    pub parameters: Box<dyn Any>,
    pub scheduling_requirements: Vec<SystemSchedulingRequirement>,
}

impl std::fmt::Debug for SystemData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System")
            .field("name", &self.name)
            .field("run", &(self.run.as_ref() as *const _))
            .field("last_run_tick", &self.last_run_tick)
            .field("parameters", &self.parameters)
            .finish()
    }
}

pub trait SystemParam {
    type Init: Any;

    fn init(world: &World) -> Self::Init;
    fn create(prepared: &mut Self::Init) -> Self;
}

impl<TData, TFilters> SystemParam for Query<TData, TFilters>
where
    TData: ToQueryInfo,
    TFilters: QueryFilter,
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
    fn into_system(self) -> SystemFn;
}

impl<F, P1> IntoSystem<P1> for F
where
    F: Fn(SystemRunContext, P1) + 'static,
    P1: SystemParam,
{
    fn init(world: &World) -> Box<dyn Any> {
        Box::new(P1::init(world))
    }

    fn into_system(self) -> SystemFn {
        Box::new(move |world, tick, parameters| {
            let context = SystemRunContext::new(world, tick);
            let parameters = parameters.downcast_mut::<P1::Init>().unwrap();
            self(context, P1::create(parameters));
        })
    }
}

macro_rules! impl_into_system {
    ($($members:ident),+) => {
        impl<
            F: Fn(SystemRunContext, $($members),+) + 'static,
            $($members: SystemParam),*
        > IntoSystem<($($members),+)> for F {
            fn init(world: &World) -> Box<dyn Any> {
                Box::new(($($members::init(world),)*))
            }

            #[allow(non_snake_case)]
            fn into_system(self) -> SystemFn {
                Box::new(move |world, tick, parameters| {
                    let context = SystemRunContext::new(world, tick);
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
