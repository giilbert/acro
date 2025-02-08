use std::{cell::RefCell, rc::Rc};

use acro_ecs::World;
use acro_math::Vec2;
use cfg_if::cfg_if;

#[cfg(not(target_arch = "wasm32"))]
use deno_core::{error::AnyError, op2};
use winit::keyboard::KeyCode;

use crate::WindowState;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[op2(fast)]
        pub fn op_get_key_press(
            #[state] world: &Rc<RefCell<World>>,
            #[string] key: &str,
        ) -> Result<bool, AnyError> {
            let world = world.borrow();
            let window_state = world.resources().get::<WindowState>();

            Ok(window_state
                .keys_pressed
                .contains(&deno_core::serde_json::from_str::<KeyCode>(key)?))
        }

        #[op2]
        #[serde]
        pub fn op_get_mouse_position(#[state] world: &Rc<RefCell<World>>) -> Result<Vec2, AnyError> {
            let world = world.borrow();
            let window_state = world.resources().get::<WindowState>();

            Ok(window_state.mouse_position)
        }

        #[op2(fast)]
        pub fn op_get_mouse_press(
            #[state] world: &Rc<RefCell<World>>,
            #[string] button: &str,
        ) -> Result<bool, AnyError> {
            let world = world.borrow();
            let window_state = world.resources().get::<WindowState>();

            Ok(window_state
                .mouse_buttons_pressed
                .contains(&deno_core::serde_json::from_str::<winit::event::MouseButton>(button)?)
                && !window_state.ui_processed_click)
        }
    } else {
        use wasm_bindgen::prelude::*;
        use acro_scripting::wasm_ops::{get_ecs_state, JsResult};

        #[wasm_bindgen]
        pub fn op_get_key_press(
            key: &str,
        ) -> JsResult<bool> {
            let (world, ..) = get_ecs_state();
            let world = world.borrow();
            let window_state = world.resources().get::<WindowState>();

            Ok(window_state
                .keys_pressed
                .contains(&serde_json::from_str::<KeyCode>(key)?))
        }

        #[wasm_bindgen]
        pub fn op_get_mouse_position() -> JsResult<JsValue> {
            let (world, ..) = get_ecs_state();
            let world = world.borrow();
            let window_state = world.resources().get::<WindowState>();

            Ok(serde_wasm_bindgen::to_value(&window_state.mouse_position)?)
        }

        #[wasm_bindgen]
        pub fn op_get_mouse_press(
            button: &str,
        ) -> JsResult<bool> {
            let (world, ..) = get_ecs_state();
            let world = world.borrow();
            let window_state = world.resources().get::<WindowState>();

            Ok(window_state
                .mouse_buttons_pressed
                .contains(&serde_json::from_str::<winit::event::MouseButton>(button)?)
                && !window_state.ui_processed_click)
        }
    }
}
