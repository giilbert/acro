use acro_assets::AssetsPlugin;
use acro_ecs::{Application, Plugin, Query, Stage, SystemRunContext, With};
use acro_log::LogPlugin;
use acro_math::{Children, GlobalTransform, MathPlugin, Parent, Root, Transform};
use acro_render::{Camera, CameraType, MainCamera, Mesh, RenderPlugin, Vertex};

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
                "crates/acro_render/src/shaders/basic-mesh.wgsl",
            ),
            GlobalTransform::default(),
            Transform::default(),
            Parent(root),
            Children(vec![]),
        ));

        world.spawn((
            Camera::new(
                CameraType::Perspective {
                    fov: 70.0,
                    near: 0.01,
                    far: 1_000.0,
                },
                800,
                600,
            ),
            MainCamera,
            GlobalTransform::default(),
            Transform {
                position: [0.0, 0.0, -2.0].into(),
                ..Default::default()
            },
            Parent(root),
            Children(vec![]),
        ));

        app.add_system(Stage::FixedUpdate, [], update);
    }
}

fn main() {
    Application::new()
        .add_plugin(LogPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(MathPlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(TestPlugin)
        .run();
}
