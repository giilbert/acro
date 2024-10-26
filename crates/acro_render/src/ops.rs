use std::{cell::RefCell, rc::Rc};

use acro_ecs::World;
use acro_math::Vec2;
use deno_core::{error::AnyError, op2};
use winit::keyboard::KeyCode;

use crate::WindowState;

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
