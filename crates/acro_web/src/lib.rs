use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    acro_ecs::Application::new().run();
}
