use std::ops::Deref;

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

use crate::state::RendererHandle;

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
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            is_dirty: true,
            vertex_buffer: None,
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
        if !<Mut<Mesh> as Deref>::deref(&mesh).is_dirty {
            continue;
        }

        println!("Uploading mesh data");
        let device = &renderer.device;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        mesh.is_dirty = false;
        mesh.vertex_buffer = Some(vertex_buffer);
    }
}

pub fn render_mesh_system(
    ctx: SystemRunContext,
    mesh_query: Query<&Mesh>,
    renderer: Res<RendererHandle>,
) {
    for mesh in mesh_query.over(&ctx) {
        let device = &renderer.device;
        let queue = &renderer.queue;
    }
}
