use acro_ecs::{Query, Res, ResMut, SystemRunContext};
use acro_reflect::Reflect;
use acro_scripting::{EventEmitter, EventQueue};
use tracing::info;

use crate::ui_element_state::UiElementState;

#[derive(Debug, Default, Reflect)]
pub struct Button {
    pub last_press_state: bool,
    pub click: EventEmitter<()>,
}

pub fn poll_button_interaction(
    ctx: SystemRunContext,
    button_query: Query<(&UiElementState, &mut Button)>,
) {
    for (state, mut button) in button_query.over(&ctx) {
        if button.last_press_state == true && !state.is_pressed {
            button.click.emit(());
        }

        button.last_press_state = state.is_pressed;
    }
}
