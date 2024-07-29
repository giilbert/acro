use std::{borrow::Cow, collections::HashMap, ops::Deref, sync::Arc};

use acro_math::Float;
use wgpu::include_wgsl;

use crate::state::RendererHandle;

#[derive(Debug)]
pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) model_matrix_buffer: wgpu::Buffer,
    pub(crate) model_matrix_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) model_matrix_bind_group: wgpu::BindGroup,
}

impl Shader {
    pub fn new(renderer: &RendererHandle, source: impl ToString) -> Self {
        let module = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(source.to_string())),
            });

        let model_matrix_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("model_matrix_buffer"),
            size: std::mem::size_of::<[[Float; 4]; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let model_matrix_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("model_matrix_bind_group_layout"),
                });

        let model_matrix_bind_group =
            renderer
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &model_matrix_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_matrix_buffer.as_entire_binding(),
                    }],
                    label: Some("model_matrix_bind_group"),
                });

        let model_matrix_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        Self {
            module,
            model_matrix_buffer,
            model_matrix_bind_group_layout,
            model_matrix_bind_group,
        }
    }
}

// TODO: Make a reusuable Handle<T> type and refactor all _Handles?
#[derive(Debug, Clone)]
pub struct ShaderHandle {
    pub name: String,
    pub(crate) inner: Arc<Shader>,
}

impl Deref for ShaderHandle {
    type Target = Shader;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct Shaders {
    renderer: RendererHandle,
    shaders: HashMap<String, ShaderHandle>,
}

impl Shaders {
    pub fn new(renderer: &RendererHandle) -> Self {
        let mut shaders = Self {
            renderer: renderer.clone(),
            shaders: HashMap::new(),
        };

        shaders.create_shader("basic-mesh", include_str!("shaders/basic-mesh.wgsl"));

        shaders
    }

    pub fn handle_by_name(&self, name: impl ToString) -> Option<&ShaderHandle> {
        self.shaders.get(&name.to_string())
    }

    pub fn create_shader(&mut self, name: impl ToString, source: impl ToString) -> ShaderHandle {
        let shader = Shader::new(&self.renderer, source);
        let handle = ShaderHandle {
            name: name.to_string(),
            inner: Arc::new(shader),
        };
        self.shaders.insert(name.to_string(), handle.clone());
        handle
    }
}
