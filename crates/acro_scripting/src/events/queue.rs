use std::{
    any::Any,
    cell::{RefCell, RefMut},
    collections::VecDeque,
    fmt::Debug,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use super::EventEmitter;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventListenerId(pub usize);

impl EventListenerId {
    pub fn next_id(&mut self) -> Self {
        let id = *self;
        self.0 += 1;
        id
    }
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
