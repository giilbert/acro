mod mesh;
mod shader;
mod state;
mod window;

pub use crate::{
    mesh::{Mesh, Vertex},
    shader::Shaders,
};

use acro_ecs::{Application, Plugin, ResMut, Stage, SystemRunContext};
use mesh::{render_mesh_system, upload_mesh_system};
use state::RendererHandle;
use window::Window;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&mut self, app: &mut Application) {
        app.world().init_component::<Mesh>();

        let window = Window::new();
        app.set_runner(move |app| {
            window.run(app);
        });

        app.add_system(Stage::PreRender, [], start_render_system);
        app.add_system(Stage::PreRender, [], upload_mesh_system);
        app.add_system(Stage::Render, [], render_mesh_system);
        app.add_system(Stage::PostRender, [], end_render_system);
    }
}

fn start_render_system(_ctx: SystemRunContext, renderer: ResMut<RendererHandle>) {
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

    *renderer.encoder.lock() = Some(encoder);
    *renderer.view.lock() = Some(view);
    *renderer.output.lock() = Some(output);
}

fn end_render_system(_ctx: SystemRunContext, renderer: ResMut<RendererHandle>) {
    let output = renderer.output.lock().take().expect("output already taken");

    let commands = renderer
        .take_encoder()
        .expect("encoder already finished for this frame")
        .finish();
    renderer.queue.submit(std::iter::once(commands));

    *renderer.view.lock() = None;
    output.present();
}
