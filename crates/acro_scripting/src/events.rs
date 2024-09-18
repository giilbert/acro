use std::{
    any::Any,
    borrow::BorrowMut,
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, VecDeque},
    fmt::Debug,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use acro_ecs::World;
use acro_reflect::{Reflect, ReflectFunctionCallError, ReflectPath as R, ReflectSetError};
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

#[derive(Debug)]
pub struct EventQueue<T: Any> {
    inner: Rc<RefCell<EventQueueInner>>,
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct AnyEventQueue {
    inner: Rc<RefCell<EventQueueInner>>,
}

#[derive(Debug, Clone)]
pub struct WeakEventQueueRef {
    inner: Weak<RefCell<EventQueueInner>>,
}

#[derive(Debug)]
pub struct EventQueueInner {
    type_id: std::any::TypeId,
    is_attached: bool,
    pub(crate) data: VecDeque<Rc<dyn Any>>,
}

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

impl<T: 'static> Default for EventQueue<T> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(EventQueueInner::new(
                std::any::TypeId::of::<T>(),
            ))),
            _phantom: PhantomData,
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

    pub fn untyped(&self) -> AnyEventQueue {
        AnyEventQueue {
            inner: Rc::clone(&self.inner),
        }
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

impl EventQueueInner {
    pub fn new(type_id: std::any::TypeId) -> Self {
        Self {
            type_id,
            is_attached: false,
            data: VecDeque::new(),
        }
    }

    #[inline(always)]
    pub fn has_events(&self) -> bool {
        !self.data.is_empty()
    }
}

impl WeakEventQueueRef {
    pub fn upgrade(&self) -> Option<AnyEventQueue> {
        self.inner.upgrade().map(|inner| AnyEventQueue { inner })
    }
}

impl AnyEventQueue {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(EventQueueInner::new(
                std::any::TypeId::of::<()>(),
            ))),
        }
    }

    pub fn into_weak(&self) -> WeakEventQueueRef {
        WeakEventQueueRef {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub fn next(&self) -> Option<Rc<dyn Any>> {
        let mut inner = self
            .inner
            .try_borrow_mut()
            .expect("inner is already borrowed");
        inner.data.pop_front()
    }

    pub fn downcast<T: 'static>(&self) -> Option<EventQueue<T>> {
        if std::any::TypeId::of::<T>() != self.inner.borrow().type_id {
            None
        } else {
            Some(EventQueue {
                inner: Rc::clone(&self.inner),
                _phantom: PhantomData,
            })
        }
    }
}

impl<T: 'static> Reflect for EventEmitter<T> {
    fn get_name(&self) -> &'static str {
        "EventEmitter<T>"
    }

    fn get_field_names(&self) -> &'static [&'static str] {
        &[]
    }

    fn get_full_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn get_opt(&self, path: &R) -> Option<&dyn Any> {
        None
    }

    fn set_any(&mut self, path: &R, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
        Err(ReflectSetError::PathNotFound)
    }

    fn call_method(
        &self,
        path: &R,
        arguments: Vec<Box<dyn Any>>,
    ) -> Result<Option<Box<dyn Any>>, ReflectFunctionCallError> {
        match path {
            R::Property("attach_javascript_function", path) if **path == R::End => {
                // let mut arguments = arguments.into_iter();
                // let arg_0 = arguments
                //     .next()
                //     .ok_or_else(|| ReflectFunctionCallError::MissingArgument(0))?
                //     .downcast()
                //     .map_err(|_| ReflectFunctionCallError::ArgumentTypeMismatch(0))?;
                // let arg_1 = arguments
                //     .next()
                //     .ok_or_else(|| ReflectFunctionCallError::MissingArgument(1))?
                //     .downcast()
                //     .map_err(|_| ReflectFunctionCallError::ArgumentTypeMismatch(1))?;

                // self.attach_javascript_function(*arg_0, *arg_1);

                Ok(None)
            }
            _ => Err(ReflectFunctionCallError::PathNotFound),
        }
    }
}

/// Registers a function as an event handler and returns a handle to it. The handle can be passed in
/// to `Reflect`ed methods to bind JavaScript functions to Rust event emitters.
#[op2]
fn create_event_listener(
    #[state] world: &Rc<RefCell<World>>,
    #[global] function: v8::Global<v8::Value>,
) -> Result<u32, deno_core::error::AnyError> {
    let world = world.borrow();
    let mut runtime = world.resource_mut::<ScriptingRuntime>();

    let event_listener_handle =
        runtime.create_event_listener_function(unsafe { Function::from_v8_unchecked(function) });

    Ok(event_listener_handle.0)
}
