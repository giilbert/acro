use std::{borrow::Cow, collections::HashMap, ops::Deref, sync::Arc};

use acro_math::{Float, Mat4};

use crate::state::RendererHandle;

#[derive(Debug)]
pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) bind_groups: HashMap<BindGroupId, BindGroupData>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum BindGroupId {
    ModelMatrix,
    ViewProjectionMatrix,
    Custom(String),
}

#[derive(Debug)]
pub struct BindGroupData {
    pub(crate) uniforms: HashMap<UniformId, UniformData>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum UniformId {
    ModelMatrix,
    ViewMatrix,
    ProjectionMatrix,
    Custom(String),
}

#[derive(Debug)]
pub struct UniformData {
    pub(crate) index: u32,
    pub(crate) stage: wgpu::ShaderStages,
    pub(crate) buffer: wgpu::Buffer,
}

impl Shader {
    pub fn new(renderer: &RendererHandle, source: impl ToString, options: ShaderOptions) -> Self {
        let module = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(source.to_string())),
            });

        let mut bind_groups = HashMap::new();
        for bind_groups_options in &options.bind_groups {
            let mut uniform_data = HashMap::new();

            for (index, uniform_options) in bind_groups_options.uniforms.iter().enumerate() {
                let (size, label) = match uniform_options.uniform_type {
                    UniformType::Mat4 => (std::mem::size_of::<Mat4>() as u64, "Mat4"),
                };

                let buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                uniform_data.insert(
                    uniform_options.id.clone(),
                    UniformData {
                        index: index as u32,
                        stage: uniform_options.stage,
                        buffer,
                    },
                );
            }

            let bind_group_layout =
                renderer
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: uniform_data
                            .values()
                            .map(|data| wgpu::BindGroupLayoutEntry {
                                binding: data.index,
                                visibility: data.stage,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            })
                            .collect::<Vec<_>>()
                            .as_slice(),
                        label: Some(
                            format!("{:?} bind group layout", bind_groups_options.id).as_str(),
                        ),
                    });

            let bind_group = renderer
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout,
                    entries: &uniform_data
                        .values()
                        .map(|data| wgpu::BindGroupEntry {
                            binding: data.index,
                            resource: data.buffer.as_entire_binding(),
                        })
                        .collect::<Vec<_>>(),
                    label: Some(format!("{:?} bind group", bind_groups_options.id).as_str()),
                });

            bind_groups.insert(
                bind_groups_options.id.clone(),
                BindGroupData {
                    uniforms: uniform_data,
                    bind_group,
                    bind_group_layout,
                },
            );
        }

        Self {
            module,
            bind_groups,
        }
    }
}

#[derive(Debug)]
pub struct ShaderOptions {
    pub bind_groups: Vec<BindGroupOptions>,
}
#[derive(Debug)]
pub struct BindGroupOptions {
    pub(crate) id: BindGroupId,
    pub(crate) uniforms: Vec<UniformOptions>,
}

#[derive(Debug)]
pub enum UniformType {
    Mat4,
}

#[derive(Debug)]
pub struct UniformOptions {
    pub(crate) id: UniformId,
    pub(crate) stage: wgpu::ShaderStages,
    pub(crate) uniform_type: UniformType,
}

impl ShaderOptions {
    pub fn mesh_defaults() -> Self {
        Self {
            bind_groups: vec![
                BindGroupOptions {
                    id: BindGroupId::ModelMatrix,
                    uniforms: vec![UniformOptions {
                        id: UniformId::ModelMatrix,
                        stage: wgpu::ShaderStages::VERTEX,
                        uniform_type: UniformType::Mat4,
                    }],
                },
                BindGroupOptions {
                    id: BindGroupId::ViewProjectionMatrix,
                    uniforms: vec![
                        UniformOptions {
                            id: UniformId::ViewMatrix,
                            stage: wgpu::ShaderStages::VERTEX,
                            uniform_type: UniformType::Mat4,
                        },
                        UniformOptions {
                            id: UniformId::ProjectionMatrix,
                            stage: wgpu::ShaderStages::VERTEX,
                            uniform_type: UniformType::Mat4,
                        },
                    ],
                },
            ],
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

        shaders.create_shader(
            "basic-mesh",
            include_str!("shaders/basic-mesh.wgsl"),
            ShaderOptions::mesh_defaults(),
        );

        shaders
    }

    pub fn handle_by_name(&self, name: impl ToString) -> Option<&ShaderHandle> {
        self.shaders.get(&name.to_string())
    }

    pub fn create_shader(
        &mut self,
        name: impl ToString,
        source: impl ToString,
        options: ShaderOptions,
    ) -> ShaderHandle {
        let shader = Shader::new(&self.renderer, source, options);
        let handle = ShaderHandle {
            name: name.to_string(),
            inner: Arc::new(shader),
        };
        self.shaders.insert(name.to_string(), handle.clone());
        handle
    }
}
