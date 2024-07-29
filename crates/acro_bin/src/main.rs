use acro_ecs::{Application, Plugin};
use acro_render::{Mesh, RenderPlugin, Shaders, Vertex};

struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&mut self, app: &mut Application) {
        let world = app.world();

        let entity = world.spawn();
        world.insert(
            entity,
            Mesh::new(
                vec![
                    Vertex {
                        position: [-0.5, -0.5, 0.0].into(),
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.0].into(),
                    },
                    Vertex {
                        position: [0.0, 0.5, 0.0].into(),
                    },
                ],
                vec![0, 1, 2],
                "basic-mesh",
            ),
        )
    }
}

fn main() {
    Application::new()
        .add_plugin(RenderPlugin)
        .add_plugin(TestPlugin)
        .run();
}
