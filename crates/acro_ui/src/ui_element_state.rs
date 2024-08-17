use acro_ecs::{Name, Query, Res, SystemRunContext};
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
    window_state: Res<WindowState>,
) {
    for (name, rect, mut state) in rect_query.over(&ctx) {
        state.is_hovered = rect.contains(window_state.mouse_position);
        state.is_pressed = state.is_hovered
            && window_state
                .mouse_buttons_pressed
                .contains(&MouseButton::Left);
    }
}
