use acro_ecs::{
    query::{Query, With},
    schedule::Stage,
    systems::SystemRunContext,
    Application, Plugin,
};
use acro_math::{Children, GlobalTransform, MathPlugin, Parent, Transform};
use acro_render::{Mesh, RenderPlugin, Vertex};

fn update(ctx: SystemRunContext, query: Query<&mut Transform, With<Mesh>>) {
    for mut transform in query.over(&ctx) {
        transform.position.x += 0.0000001;
    }
}

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
        );

        world.insert(entity, GlobalTransform::default());
        world.insert(entity, Transform::default());
        world.insert(entity, Parent(entity));
        world.insert(entity, Children(vec![]));

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
