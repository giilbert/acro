use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use fnv::FnvHashMap;

use crate::{platform::FunctionHandle, ScriptingRuntime};

use super::{AnyEventQueue, EventListenerId, WeakEventQueueRef};

#[derive(Default, Clone)]
pub struct EventListenerStore {
    inner: Rc<RefCell<EventListenerStoreInner>>,
}

impl EventListenerStore {
    pub fn inner(&self) -> Ref<EventListenerStoreInner> {
        self.inner.borrow()
    }

    pub fn inner_mut(&self) -> RefMut<EventListenerStoreInner> {
        self.inner.borrow_mut()
    }
}

struct BoundEventQueue {
    pub queue: WeakEventQueueRef,
    pub function: FunctionHandle,
}

#[derive(Default)]
pub struct EventListenerStoreInner {
    event_listener_id: EventListenerId,
    event_listeners: FnvHashMap<EventListenerId, BoundEventQueue>,
}

impl EventListenerStoreInner {
    pub fn create_event_listener_function(&mut self, function: FunctionHandle) -> AnyEventQueue {
        let queue = AnyEventQueue::new();
        let id = self.event_listener_id.next_id();

        let weak_ref = queue.into_weak();
        self.event_listeners.insert(
            id,
            BoundEventQueue {
                queue: weak_ref,
                function,
            },
        );

        queue
    }

    pub fn remove_event_listener(&mut self, id: EventListenerId) {}

    pub fn update_active_event_listeners(
        &mut self,
        runtime: &mut ScriptingRuntime,
    ) -> eyre::Result<()> {
        let mut dead_listeners = vec![];

        for (id, bound_queue) in &self.event_listeners {
            match bound_queue.queue.upgrade() {
                Some(queue) => {
                    while let Some(data) = queue.next() {
                        // TODO: call function with data
                        bound_queue.function.call::<()>(runtime, &[()])?;
                    }
                }
                None => dead_listeners.push(*id),
            }
        }

        // remove dead listeners
        for id in dead_listeners {
            self.event_listeners.remove(&id);
        }

        Ok(())
    }
}
