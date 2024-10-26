use std::{any::Any, collections::HashMap, fmt::Debug, rc::Rc};

use crate::function::FunctionHandle;
use acro_reflect::{Reflect, ReflectFunctionCallError, ReflectPath as R, ReflectSetError};

use super::{queue::EventQueue, EventListenerId};

#[derive(Debug, Default)]
pub struct EventEmitter<T: 'static> {
    listener_id: usize,
    listeners: HashMap<EventListenerId, EventQueue<T>>,
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

    pub fn attach_javascript_function(&mut self, function: FunctionHandle) {
        let queue = function.into_event_queue();
        self.attach(queue.downcast().expect("downcast failed"));
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
        &mut self,
        path: &R,
        arguments: Vec<Box<dyn Any>>,
    ) -> Result<Option<Box<dyn Any>>, ReflectFunctionCallError> {
        match path {
            R::Property("bind", path) if **path == R::End => {
                let mut arguments = arguments.into_iter();
                let arg_0: Box<FunctionHandle> = arguments
                    .next()
                    .ok_or_else(|| ReflectFunctionCallError::MissingArgument(0))?
                    .downcast()
                    .map_err(|_| ReflectFunctionCallError::ArgumentTypeMismatch(0))?;

                self.attach_javascript_function(*arg_0);

                Ok(None)
            }
            _ => Err(ReflectFunctionCallError::PathNotFound),
        }
    }
}
