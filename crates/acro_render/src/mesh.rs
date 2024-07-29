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
use acro_math::{Float, Vec3};
use bytemuck::{Pod, Zeroable};
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use crate::{
    shader::{ShaderHandle, Shaders},
    state::RendererHandle,
};

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
    pub(crate) render_pipeline: Option<wgpu::RenderPipeline>,
    pub(crate) shader_name: String,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, shader_name: impl ToString) -> Self {
        Self {
            vertices,
            indices,
            is_dirty: true,
            vertex_buffer: None,
            index_buffer: None,
            render_pipeline: None,
            shader_name: shader_name.to_string(),
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
    shaders: Res<Shaders>,
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

        let shader = shaders
            .handle_by_name(&mesh.shader_name)
            .expect("shader not found");
        let module = &shader.module;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VERTEX_FORMAT,
                    }],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        mesh.is_dirty = false;
        mesh.vertex_buffer = Some(vertex_buffer);
        mesh.index_buffer = Some(index_buffer);
        mesh.render_pipeline = Some(render_pipeline);
    }
}

pub fn render_mesh_system(
    ctx: SystemRunContext,
    mesh_query: Query<&Mesh>,
    renderer: Res<RendererHandle>,
    shaders: Res<Shaders>,
) {
    let view = renderer.view();
    let mut encoder = renderer.encoder();

    let mut mesh_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    for mesh in mesh_query.over(&ctx) {
        mesh_render_pass.set_pipeline(&mesh.render_pipeline.as_ref().expect("no render pipeline"));
        mesh_render_pass.set_vertex_buffer(0, mesh.vertex_buffer.as_ref().unwrap().slice(..));
        mesh_render_pass.set_index_buffer(
            mesh.index_buffer.as_ref().unwrap().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        mesh_render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
    }
}
