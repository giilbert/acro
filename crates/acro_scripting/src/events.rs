use std::{
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
    collections::{HashMap, VecDeque},
    fmt::Debug,
    rc::Rc,
};

use deno_core::{op2, v8, v8::HandleScope, OpState};
use rustyscript::js_value::Function;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventListenerId(usize);

#[derive(Debug, Default)]
pub struct EventEmitter<T> {
    listener_id: usize,
    listeners: HashMap<EventListenerId, EventListener<T>>,
}

#[derive(Debug, Default)]
pub struct EventQueue<T> {
    inner: Rc<RefCell<EventQueueInner<T>>>,
}

#[derive(Debug, Default)]
pub struct EventQueueInner<T> {
    is_attached: bool,
    data: VecDeque<Rc<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptableEventListenerId(usize);

#[derive(Debug)]
pub enum EventListener<T> {
    Native(EventQueue<T>),
    Scriptable(Function),
}

impl<T> EventEmitter<T> {
    fn next_id(&mut self) -> EventListenerId {
        let id = self.listener_id;
        self.listener_id += 1;
        EventListenerId(id)
    }

    pub fn attach_event_queue(&mut self, queue: EventQueue<T>) -> EventListenerId {
        let id = self.next_id();
        self.listeners.insert(id, EventListener::Native(queue));
        id
    }

    pub fn emit(&self, data: T) {
        let data = Rc::new(data);

        for (_id, listener) in &self.listeners {
            match listener {
                EventListener::Native(queue) => queue.inner_mut().data.push_back(data.clone()),
                _ => todo!(),
            }
        }
    }
}

impl<T> Clone for EventQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> EventQueue<T> {
    pub fn inner_mut(&self) -> RefMut<EventQueueInner<T>> {
        RefCell::borrow_mut(&self.inner)
    }

    pub fn attach_if_not_attached(&self, emitter: &mut EventEmitter<T>) {
        let mut inner = self.inner_mut();
        if !inner.is_attached {
            emitter.attach_event_queue(self.clone());
            inner.is_attached = true;
        }
    }

    pub fn next(&self) -> Option<Rc<T>> {
        let mut inner = self.inner_mut();
        inner.data.pop_front()
    }
}

#[op2]
fn register_event_listener(#[global] function: v8::Global<v8::Function>) {}
