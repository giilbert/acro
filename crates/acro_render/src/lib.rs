mod state;
mod window;

use acro_ecs::{Application, Plugin};
use window::Window;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&mut self, app: &mut Application) {
        let window = Window::new();
        app.set_runner(move |mut app| {
            window.run(app);
        });
    }
}
