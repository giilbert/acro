use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    rc::Rc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventListenerId(usize);

#[derive(Debug, Default)]
pub struct EventEmitter<T> {
    listener_id: usize,
    listeners: HashMap<EventListenerId, EventListener<T>>,
}

#[derive(Debug, Default)]
pub struct EventQueue<T> {
    data: VecDeque<Rc<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptableEventListenerId(usize);

#[derive(Debug)]
pub enum EventListener<T> {
    Native(EventQueue<T>),
    Scriptable(ScriptableEventListenerId),
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

    pub fn emit(&mut self, data: T) {
        let data = Rc::new(data);

        for (_id, mut listener) in &mut self.listeners {
            match &mut listener {
                EventListener::Native(queue) => queue.data.push_back(data.clone()),
                _ => todo!(),
            }
        }
    }
}
