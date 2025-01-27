#[cfg(target_arch = "wasm32")]
use acro_scripting::wasm_ops;
use cfg_if::cfg_if;
use std::{cell::RefCell, rc::Rc};

use acro_ecs::{EntityId, World};
#[cfg(not(target_arch = "wasm32"))]
use deno_core::op2;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::tree::WorldTreeExt;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[op2]
        #[serde]
        pub fn op_get_entity_by_absolute_path(
            #[state] world: &Rc<RefCell<World>>,
            #[string] path: &str,
        ) -> Option<EntityId> {
            world.borrow().get_entity_by_absolute_path(&path)
        }
    } else {
        #[wasm_bindgen]
        pub fn op_get_entity_by_absolute_path(path: &str) -> Result<JsValue, JsError> {
            let (world, ..) = wasm_ops::get_ecs_state();
            match world.borrow().get_entity_by_absolute_path(&path) {
                Some(entity_id) => {
                    Ok(serde_wasm_bindgen::to_value(&entity_id).map_err(wasm_ops::into_js_error)?)
                }
                None => Ok(JsValue::null()),
            }
        }
    }
}
