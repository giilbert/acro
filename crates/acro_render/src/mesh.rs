use acro_assets::Assets;
use acro_ecs::{Changed, EntityId, Query, Res, SystemRunContext, With};
use acro_math::{GlobalTransform, Vec2, Vec3};
use bytemuck::{Pod, Zeroable};
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use tracing::info;
use wgpu::util::DeviceExt;

use crate::{
    camera::MainCamera,
    mesh_geometry::{MeshGeometryData, Vertex},
    shader::{BindGroupId, Shader, UniformId},
    texture::Texture,
    Camera, RendererHandle,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    // pub vertices: Vec<Vertex>,
    // pub indices: Vec<u32>,
    pub geometry: MeshGeometryData,
    pub(crate) shader_path: String,
    pub diffuse_texture: Option<String>,
    #[serde(skip)]
    pub(crate) data: Option<MeshData>,
}

#[derive(Debug)]
pub(crate) struct MeshData {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
}

impl Mesh {
    pub fn new(
        geometry: MeshGeometryData,
        diffuse_texture: Option<impl Into<String>>,
        shader_path: impl ToString,
    ) -> Self {
        Self {
            geometry,
            diffuse_texture: diffuse_texture.map(Into::into),
            shader_path: shader_path.to_string(),
            data: None,
        }
    }
}

// TODO:
// Should mesh buffer data be uploaded to the GPU in a synchronous or asynchronous manner?
// Should the mesh buffer data be stored separately from the component?
pub fn upload_mesh_system(
    ctx: SystemRunContext,
    mesh_query: Query<(EntityId, &mut Mesh), Changed<Mesh>>,
    renderer: Res<RendererHandle>,
    assets: Res<Assets>,
) {
    for (entity, mut mesh) in mesh_query.over(&ctx) {
        let device = &renderer.device;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.geometry.vertices(&assets)),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.geometry.indices(&assets)),
            usage: wgpu::BufferUsages::INDEX,
        });

        let shader = assets.get::<Shader>(&mesh.shader_path);
        shader.notify_changes::<Mesh>(&ctx, entity);

        let module = &shader.module;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &shader
                        .bind_groups
                        .get(&BindGroupId::ModelMatrix)
                        .expect("model matrix bind group not found")
                        .bind_group_layout,
                    &shader
                        .bind_groups
                        .get(&BindGroupId::ViewProjectionMatrix)
                        .expect("view-projection matrix bind group not found")
                        .bind_group_layout,
                    &shader
                        .bind_groups
                        .get(&BindGroupId::DiffuseTexture)
                        .expect("diffuse texture bind group not found")
                        .bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.config.borrow().format,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        mesh.data = Some(MeshData {
            vertex_buffer,
            index_buffer,
            render_pipeline,
        });
    }
}

pub fn render_mesh_system(
    ctx: SystemRunContext,
    mesh_query: Query<(&GlobalTransform, &Mesh)>,
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    renderer: Res<RendererHandle>,
    assets: Res<Assets>,
) {
    let frame_state = renderer.frame_state();
    let view = &frame_state.view;
    let mut encoder = frame_state.encoder.borrow_mut();

    let (camera_transform, camera) = camera_query.single(&ctx);

    for (global_transform, mesh) in mesh_query.over(&ctx) {
        let data = mesh.data.as_ref().expect("mesh data not loaded");
        let shader = assets.get::<Shader>(&mesh.shader_path);

        {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &renderer.depth_stencil_view.borrow(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            mesh_render_pass.set_pipeline(&data.render_pipeline);

            // Update model matrix uniform
            let model_matrix_bind_group = shader
                .bind_groups
                .get(&BindGroupId::ModelMatrix)
                .expect("model matrix bind group not found");
            let model_matrix_uniform = model_matrix_bind_group
                .uniforms
                .get(&UniformId::ModelMatrix)
                .expect("model matrix uniform not found");
            renderer.queue.write_buffer(
                &model_matrix_uniform.data_type.assert_buffer(),
                0,
                bytemuck::cast_slice(global_transform.matrix.as_slice()),
            );
            mesh_render_pass.set_bind_group(0, &model_matrix_bind_group.bind_group, &[]);

            // Update view-projection matrix uniform
            let view_projection_matrix_bind_group = shader
                .bind_groups
                .get(&BindGroupId::ViewProjectionMatrix)
                .expect("view-projection matrix bind group not found");
            let view_matrix_uniform = view_projection_matrix_bind_group
                .uniforms
                .get(&UniformId::ViewMatrix)
                .expect("view matrix uniform not found");
            renderer.queue.write_buffer(
                &view_matrix_uniform.data_type.assert_buffer(),
                0,
                bytemuck::cast_slice(camera_transform.matrix.as_slice()),
            );
            let projection_matrix_uniform = view_projection_matrix_bind_group
                .uniforms
                .get(&UniformId::ProjectionMatrix)
                .expect("projection matrix uniform not found");
            renderer.queue.write_buffer(
                &projection_matrix_uniform.data_type.assert_buffer(),
                0,
                bytemuck::cast_slice(camera.projection_matrix.as_slice()),
            );
            mesh_render_pass.set_bind_group(1, &view_projection_matrix_bind_group.bind_group, &[]);

            let texture_bind_group = shader
                .bind_groups
                .get(&BindGroupId::DiffuseTexture)
                .expect("diffuse texture bind group not found");
            mesh_render_pass.set_bind_group(2, &texture_bind_group.bind_group, &[]);

            mesh_render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
            mesh_render_pass
                .set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            mesh_render_pass.draw_indexed(0..mesh.geometry.indices(&assets).len() as u32, 0, 0..1);
        }
    }
}
