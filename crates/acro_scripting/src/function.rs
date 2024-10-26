use rustyscript::js_value::Function;

use crate::{EventListenerStore, ScriptingRuntime};

pub struct FunctionHandle {
    pub(crate) function: Function,
    event_listener_store: EventListenerStore,
}

impl FunctionHandle {
    pub fn new(event_listener_store: &EventListenerStore, function: Function) -> Self {
        Self {
            function,
            event_listener_store: event_listener_store.clone(),
        }
    }

    pub fn into_event_queue(self) -> crate::AnyEventQueue {
        self.event_listener_store
            .inner_mut()
            .create_event_listener_function(self.function)
    }
}
