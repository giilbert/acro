use acro_ecs::{Query, Res, SystemRunContext};
use acro_scripting::{EventEmitter, EventQueue};
use tracing::info;

use crate::ui_element_state::UiElementState;

#[derive(Debug, Default)]
pub struct ButtonEvents {
    click: EventEmitter<()>,
}

#[derive(Debug, Default)]
pub struct Button {
    pub last_press_state: bool,
    pub events: ButtonEvents,
}

pub fn poll_button_interaction(
    ctx: SystemRunContext,
    button_query: Query<(&UiElementState, &mut Button)>,
) {
    for (state, mut button) in button_query.over(&ctx) {
        if button.last_press_state == true && !state.is_pressed {
            info!("button pressed released");
            button.events.click.emit(());
        }

        button.last_press_state = state.is_pressed;
    }
}

#[derive(Default)]
pub struct ButtonClickTestQueue(pub EventQueue<()>);

pub fn handle_button_click_test(
    ctx: SystemRunContext,
    button_click_queue: Res<ButtonClickTestQueue>,
) {
}
