use acro_ecs::{Name, Query, Res, ResMut, SystemRunContext};
use acro_render::WindowState;
use tracing::info;
use winit::event::MouseButton;

use crate::rect::Rect;

#[derive(Debug, Default)]
pub struct UiElementState {
    pub is_hovered: bool,
    pub is_pressed: bool,
}

pub fn poll_ui_element_state(
    ctx: SystemRunContext,
    rect_query: Query<(&Name, &Rect, &mut UiElementState)>,
    mut window_state: ResMut<WindowState>,
) {
    let mut ui_processed_click = false;

    for (name, rect, mut state) in rect_query.over(&ctx) {
        state.is_hovered = rect.contains(window_state.mouse_position);
        state.is_pressed = state.is_hovered
            && window_state
                .mouse_buttons_pressed
                .contains(&MouseButton::Left);

        if state.is_pressed {
            ui_processed_click = true;
        }
    }

    window_state.ui_processed_click = ui_processed_click;
}
