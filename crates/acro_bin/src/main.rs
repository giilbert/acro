use acro_ecs::{
    entity::EntityId,
    query::{Query, With},
    schedule::Stage,
    systems::SystemRunContext,
    Application, Plugin,
};
use acro_math::{Children, GlobalTransform, MathPlugin, Parent, Root, Transform};
use acro_render::{Mesh, RenderPlugin, Vertex};

fn update(ctx: SystemRunContext, query: Query<&mut Transform, With<Mesh>>) {
    for mut transform in query.over(&ctx) {
        transform.position.x += 0.00001;
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

        app.add_system(Stage::Update, [], update);
    }
}

fn main() {
    Application::new()
        .add_plugin(MathPlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(TestPlugin)
        .run();
}
