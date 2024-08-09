use acro_math::Vec2;
use acro_render::RendererHandle;
use deno_core::futures::SinkExt;
use tracing::info;
use wgpu::{
    core::instance, util::DeviceExt, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    VertexAttribute,
};

#[rustfmt::skip]
const BOX_VERTICES: &[f32] = &[
    // 0.0, 0.0,
    // 1.0, 0.0,
    // 1.0, 1.0,
    // 0.0, 1.0,
    
    0.0, 0.0,
    0.0, -1.0,
    1.0, -1.0,
    1.0, 0.0,
];
const BOX_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

#[derive(Debug, Clone, Copy)]
struct InstanceData {
    offset: Vec2,
    size: Vec2,
}

impl InstanceData {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec2>() as u64,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Zeroable for InstanceData {}
unsafe impl bytemuck::Pod for InstanceData {}

pub struct BoxRenderer {
    instance_data: Vec<InstanceData>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    screen_size_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,

    shader: wgpu::ShaderModule,

    context_bind_group: wgpu::BindGroup,
    context_bind_group_layout: wgpu::BindGroupLayout,
}

impl BoxRenderer {
    pub fn new(renderer: &RendererHandle) -> Self {
        // let instance_data = Vec::<InstanceData>::with_capacity(128);
        // instance_data.push(InstanceData {
        //     offset: Vec2::new(50.0, 100.0),
        //     size: Vec2::new(100.0, 100.0),
        // });

        let device = &renderer.device;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BoxRenderer Vertex Buffer"),
            contents: bytemuck::cast_slice(BOX_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BoxRenderer Index Buffer"),
            contents: bytemuck::cast_slice(BOX_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // TODO: handle resizing the number of instances

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("BoxRenderer Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/box.wgsl").into()),
        });

        let screen_size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("BoxRenderer Context Buffer"),
            size: std::mem::size_of::<Vec2>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let context_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("BoxRenderer Context Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: wgpu::ShaderStages::VERTEX,
                }],
            });

        let context_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("BoxRenderer Context Bind Group"),
            layout: &context_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &screen_size_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&context_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        attributes: &[VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                        array_stride: std::mem::size_of::<Vec2>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                    },
                    InstanceData::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.config.borrow().format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            cache: None,
        });

        Self {
            instance_data: Vec::new(),
            vertex_buffer,
            index_buffer,
            screen_size_buffer,
            render_pipeline,
            shader,
            context_bind_group,
            context_bind_group_layout,
        }
    }

    pub fn queue(&mut self, offset: Vec2, size: Vec2) {
        // info!("drawing box at {:?} with size {:?}", offset, size);
        self.instance_data.push(InstanceData { offset, size });
    }

    pub fn finish(&mut self, renderer: &RendererHandle) -> eyre::Result<()> {
        // info!(
        //     "finishing box renderer with {} boxes",
        //     self.instance_data.len()
        // );

        let instance_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("BoxRenderer Instance Buffer"),
                    contents: bytemuck::cast_slice(&self.instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let frame_state = renderer.frame_state();
        let view = &frame_state.view;
        let mut encoder = frame_state.encoder.borrow_mut();
        let size = renderer.size.borrow();

        let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Ui Render Pass"),
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

        ui_render_pass.set_pipeline(&self.render_pipeline);
        ui_render_pass.set_bind_group(0, &self.context_bind_group, &[]);
        renderer.queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
        );
        ui_render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        ui_render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        ui_render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        ui_render_pass.draw_indexed(
            0..BOX_INDICES.len() as _,
            0,
            0..self.instance_data.len() as _,
        );

        self.instance_data.clear();

        Ok(())
    }
}
