use std::{
    hint::black_box,
    ops::{Deref, DerefMut},
};

use acro_ecs::{
    pointer::change_detection::Mut,
    query::{Changed, Query},
    resource::Res,
    systems::SystemRunContext,
};
use acro_math::Vec3;
use bytemuck::{Pod, Zeroable};
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use crate::{shader::Shaders, state::RendererHandle};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
}

cfg_if! {
    if #[cfg(feature = "double-precision")] {
        const VERTEX_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float64x3;
    } else {
        const VERTEX_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x3;
    }
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VERTEX_FORMAT,
            }],
        }
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub(crate) is_dirty: bool,
    pub(crate) vertex_buffer: Option<wgpu::Buffer>,
    pub(crate) index_buffer: Option<wgpu::Buffer>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            is_dirty: true,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}

// TODO:
// Should mesh buffer data be uploaded to the GPU in a synchronous or asynchronous manner?
// Should the mesh buffer data be stored separately from the component?
pub fn upload_mesh_system(
    ctx: SystemRunContext,
    mesh_query: Query<&mut Mesh, Changed<Mesh>>,
    renderer: Res<RendererHandle>,
) {
    for mut mesh in mesh_query.over(&ctx) {
        let device = &renderer.device;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        mesh.is_dirty = false;
        mesh.vertex_buffer = Some(vertex_buffer);
        mesh.index_buffer = Some(index_buffer);
    }
}

pub fn render_mesh_system(
    ctx: SystemRunContext,
    mesh_query: Query<&mut Mesh>,
    renderer: Res<RendererHandle>,
    shaders: Res<Shaders>,
) {
    let render_pass = renderer
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
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
