use std::any::Any;

use acro_assets::{Assets, AssetsPlugin};
use acro_ecs::{Application, Plugin, Query, Res, Stage, SystemRunContext, With};
use acro_log::LogPlugin;
use acro_math::{Children, GlobalTransform, MathPlugin, Parent, Root, Transform};
use acro_render::{
    Camera, CameraType, MainCamera, Mesh, RenderPlugin, Texture, Vertex, WindowState,
};
use acro_scripting::{Behavior, ScriptingPlugin, SourceFile};
use tracing::info;

fn update(
    ctx: SystemRunContext,
    query: Query<&mut Transform, With<Mesh>>,
    window: Res<WindowState>,
) {
    let mut transform = query.single(&ctx);
    transform.position.x = window.mouse_position.x / 100.0;
    transform.position.y = window.mouse_position.y / 100.0;
}

struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&mut self, app: &mut Application) {
        let mut world = app.world();

        {
            let assets = world.resources().get::<Assets>();
            assets.queue::<SourceFile>("crates/acro_scripting/src/js/test.js");
        }

        let root = world.spawn((Root, GlobalTransform::default(), Transform::default()));

        world.spawn((
            Mesh::new(
                vec![
                    Vertex {
                        position: [-1.0, -1.0, 0.0].into(),
                        tex_coords: [0.0, 1.0].into(),
                    },
                    Vertex {
                        position: [1.0, -1.0, 0.0].into(),
                        tex_coords: [1.0, 1.0].into(),
                    },
                    Vertex {
                        position: [1.0, 1.0, 0.0].into(),
                        tex_coords: [1.0, 0.0].into(),
                    },
                    Vertex {
                        position: [-1.0, 1.0, 0.0].into(),
                        tex_coords: [0.0, 0.0].into(),
                    },
                ],
                vec![0, 1, 2, 0, 2, 3],
                Some("crates/acro_render/src/textures/ferris.png"),
                "crates/acro_render/src/shaders/basic-mesh.wgsl",
            ),
            GlobalTransform::default(),
            Transform::default(),
            Parent(root),
            Children(vec![]),
            Behavior::new("crates/acro_scripting/src/js/test.js"),
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
                position: [0.0, 0.0, -20.0].into(),
                ..Default::default()
            },
            Parent(root),
            Children(vec![]),
        ));

        drop(world);
        app.add_system(Stage::FixedUpdate, [], update);
    }
}

fn main() {
    Application::new()
        .add_plugin(LogPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(ScriptingPlugin)
        .add_plugin(MathPlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(TestPlugin)
        .run();
}
