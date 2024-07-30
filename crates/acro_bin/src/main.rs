use acro_ecs::{Application, Plugin, Query, Stage, SystemRunContext, With};
use acro_log::LogPlugin;
use acro_math::{Children, GlobalTransform, MathPlugin, Parent, Root, Transform};
use acro_render::{Mesh, RenderPlugin, Vertex};

fn update(ctx: SystemRunContext, query: Query<&mut Transform, With<Mesh>>) {
    for mut transform in query.over(&ctx) {
        transform.position.x += 0.001;
    }
}

struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&mut self, app: &mut Application) {
        let world = app.world();

        let root = world.spawn((Root, GlobalTransform::default(), Transform::default()));

        world.spawn((
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
            GlobalTransform::default(),
            Transform::default(),
            Parent(root),
            Children(vec![]),
        ));

        app.add_system(Stage::FixedUpdate, [], update);
    }
}

fn main() {
    Application::new()
        .add_plugin(LogPlugin)
        .add_plugin(MathPlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(TestPlugin)
        .run();
}
