use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use rustyscript::js_value::Function;

use crate::{EventListenerStore, ScriptingRuntime};

pub struct FunctionHandle {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) function: Function,
    #[cfg(target_arch = "wasm32")]
    pub(crate) function: js_sys::Function,

    event_listener_store: EventListenerStore,
}

impl FunctionHandle {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native(event_listener_store: &EventListenerStore, function: Function) -> Self {
        Self {
            function,
            event_listener_store: event_listener_store.clone(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_browser(
        event_listener_store: &EventListenerStore,
        function: js_sys::Function,
    ) -> Self {
        Self {
            function,
            event_listener_store: event_listener_store.clone(),
        }
    }

    pub fn into_event_queue(self) -> crate::AnyEventQueue {
        self.event_listener_store
            .clone()
            .inner_mut()
            .create_event_listener_function(self)
    }

    pub fn call<T: serde::de::DeserializeOwned>(
        &self,
        runtime: &mut ScriptingRuntime,
        arguments: &impl Serialize,
    ) -> eyre::Result<T> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return Ok(self.function.call(runtime.inner_mut(), None, arguments)?);
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.function
                .call1(&JsValue::NULL, &serde_wasm_bindgen::to_value(arguments)?)
                .unwrap();
        };
    }
}
