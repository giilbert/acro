mod camera;
mod mesh;
mod ops;
mod shader;
mod state;
mod texture;
mod window;

use std::cell::RefCell;

pub use crate::{
    camera::{Camera, CameraType, MainCamera},
    mesh::{Mesh, Vertex},
    texture::Texture,
    window::WindowState,
};

use acro_assets::{AssetLoader, Assets};
use acro_ecs::{Application, Plugin, Res, Stage, SystemRunContext};
use acro_scene::ComponentLoaders;
use acro_scripting::ScriptingRuntime;
use camera::{update_projection_matrix, CameraOptions};
use mesh::{render_mesh_system, upload_mesh_system};
use ops::op_get_key_press;
use shader::Shader;
use state::{FrameState, RendererHandle};
use tracing::info;
use window::Window;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&mut self, app: &mut Application) {
        app.world().init_component::<Mesh>();
        app.world().init_component::<Camera>();
        app.world().init_component::<MainCamera>();

        {
            let world = app.world();
            let mut assets = world.resources().get_mut::<Assets>();
            assets.register_loader::<Shader>();
            assets.register_loader::<Texture>();

            // assets.queue::<Texture>("crates/acro_render/src/textures/ferris.png");
            // assets.queue::<Shader>("crates/acro_render/src/shaders/basic-mesh.wgsl");

            let loaders = world.resources().get_mut::<ComponentLoaders>();
            loaders.register("Mesh", |world, entity, serialized| {
                let mesh_data = serialized.into_rust::<Mesh>()?;

                world
                    .resources()
                    .get_mut::<Assets>()
                    .queue::<Shader>(&mesh_data.shader_path);
                if let Some(diffuse_texture) = &mesh_data.diffuse_texture {
                    world
                        .resources()
                        .get_mut::<Assets>()
                        .queue::<Texture>(diffuse_texture);
                }

                world.insert(entity, mesh_data);
                Ok(())
            });
            loaders.register("Camera", |world, entity, serialized| {
                let options = serialized.into_rust::<CameraOptions>()?;
                world.insert(entity, Camera::new(options.get_camera_type()?, 800, 600));
                if options.is_main_camera {
                    world.insert(entity, MainCamera);
                }

                Ok(())
            });

            let mut runtime = world.resources().get_mut::<ScriptingRuntime>();
            runtime.add_op(op_get_key_press());
        }

        let window = Window::new();
        app.set_runner(move |app| {
            window.run(app);
        });

        app.add_system(Stage::PreRender, [], start_render_system);
        app.add_system(Stage::PreRender, [], upload_mesh_system);
        app.add_system(Stage::PreRender, [], update_projection_matrix);
        app.add_system(Stage::Render, [], render_mesh_system);
        app.add_system(Stage::PostRender, [], end_render_system);
    }
}

fn start_render_system(_ctx: SystemRunContext, renderer: Res<RendererHandle>) {
    let output = renderer.surface.get_current_texture().unwrap();
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        let _clear_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.05,
                        b: 0.05,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    *renderer.frame_state.borrow_mut() = Some(FrameState {
        encoder: RefCell::new(encoder),
        view,
        output,
    });
}

fn end_render_system(_ctx: SystemRunContext, renderer: Res<RendererHandle>) {
    let frame_state = renderer.take_frame_state().expect("frame already ended");
    let encoder = frame_state.encoder.into_inner();

    let commands = encoder.finish();
    renderer.queue.submit(std::iter::once(commands));

    frame_state.output.present();
}
