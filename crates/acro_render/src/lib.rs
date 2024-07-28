mod mesh;
mod state;
mod window;

pub use mesh::{Mesh, Vertex};

use acro_ecs::{schedule::Stage, Application, Plugin};
use mesh::upload_mesh_system;
use window::Window;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&mut self, app: &mut Application) {
        app.world().init_component::<Mesh>();

        let window = Window::new();
        app.set_runner(move |mut app| {
            window.run(app);
        });

        app.add_system(Stage::PreRender, [], upload_mesh_system);
    }
}
