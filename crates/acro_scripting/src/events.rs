use std::{
    any::Any,
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
    collections::{HashMap, VecDeque},
    fmt::Debug,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use acro_ecs::World;
use deno_core::{op2, v8, v8::HandleScope, OpState};
use rustyscript::js_value::Function;
use tracing::info;

use crate::ScriptingRuntime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventListenerId(usize);

#[derive(Debug, Default)]
pub struct EventEmitter<T: 'static> {
    listener_id: usize,
    listeners: HashMap<EventListenerId, EventQueue<T>>,
}

#[derive(Debug, Default)]
pub struct EventQueue<T: Any> {
    inner: Rc<RefCell<EventQueueInner>>,
    _phantom: PhantomData<T>,
}

pub struct WeakEventQueueRef {
    inner: Weak<RefCell<EventQueueInner>>,
}

#[derive(Debug, Default)]
pub struct EventQueueInner {
    is_attached: bool,
    data: VecDeque<Rc<dyn Any>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptableEventListenerId(usize);

impl<T> EventEmitter<T> {
    fn next_id(&mut self) -> EventListenerId {
        let id = self.listener_id;
        self.listener_id += 1;
        EventListenerId(id)
    }

    pub fn attach(&mut self, queue: EventQueue<T>) -> EventListenerId {
        let id = self.next_id();
        self.listeners.insert(id, queue);
        id
    }

    pub fn emit(&self, data: T) {
        let data = Rc::new(data);

        for (_id, queue) in &self.listeners {
            queue.inner_mut().data.push_back(data.clone());
        }
    }
}

impl<T: 'static> Clone for EventQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> EventQueue<T> {
    pub fn inner_mut(&self) -> RefMut<EventQueueInner> {
        RefCell::borrow_mut(&self.inner)
    }

    pub fn attach_if_not_attached(&self, emitter: &mut EventEmitter<T>) {
        let mut inner = self.inner_mut();
        if !inner.is_attached {
            emitter.attach(self.clone());
            inner.is_attached = true;
        }
    }

    pub fn next(&self) -> Option<Rc<T>> {
        let mut inner = self.inner_mut();
        inner
            .data
            .pop_front()
            .map(|value| value.downcast().expect("inner data does not match"))
    }
}

/// Registers a function as an event handler and returns a handle to it. The handle can be passed in
/// to `Reflect`ed methods to bind JavaScript functions to Rust event emitters.
#[op2]
fn create_event_listener(
    #[state] world: &Rc<RefCell<World>>,
    #[global] function: v8::Global<v8::Function>,
) -> Result<u32, deno_core::error::AnyError> {
    let world = world.borrow();
    let runtime = world.resource_mut::<ScriptingRuntime>();

    todo!();
}
