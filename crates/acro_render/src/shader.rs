use std::{borrow::Cow, collections::HashMap, ops::Deref, sync::Arc};

use acro_assets::Loadable;
use acro_ecs::World;
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
    DiffuseTexture,
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
    Texture2D,
    Sampler,
    Custom(String),
}

#[derive(Debug)]
pub struct UniformData {
    pub(crate) index: u32,
    pub(crate) stage: wgpu::ShaderStages,
    pub(crate) data: UniformDataType,
}

#[derive(Debug)]
pub enum UniformDataType {
    Buffer(wgpu::Buffer),
    Texture(wgpu::Texture),
    Sampler(wgpu::Sampler),
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
                    UniformType::Sampler => continue,
                    UniformType::Texture2D => continue,
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
                        data: UniformDataType::Buffer(buffer),
                    },
                );
            }

            let bind_group_layout =
                renderer
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: uniform_data
                            .values()
                            .map(|data| match bind_groups_options.id {
                                BindGroupId::ModelMatrix
                                | BindGroupId::ViewProjectionMatrix
                                | BindGroupId::Custom(_) => wgpu::BindGroupLayoutEntry {
                                    binding: data.index,
                                    visibility: data.stage,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                    },
                                    count: None,
                                },
                                BindGroupId::DiffuseTexture => wgpu::BindGroupLayoutEntry {
                                    binding: data.index,
                                    visibility: data.stage,
                                    ty: match data.index {
                                        0 => wgpu::BindingType::Texture {
                                            multisampled: false,
                                            view_dimension: wgpu::TextureViewDimension::D2,
                                            sample_type: wgpu::TextureSampleType::Float {
                                                filterable: true,
                                            },
                                        },
                                        1 => wgpu::BindingType::Sampler(
                                            wgpu::SamplerBindingType::Filtering,
                                        ),
                                        _ => unreachable!(),
                                    },
                                    count: None,
                                },
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
                            resource: data.data.as_entire_binding(),
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

impl Loadable for Shader {
    fn load(world: &World, data: Vec<u8>) -> Result<Self, ()> {
        let renderer = world.resources().get::<RendererHandle>();
        Ok(Shader::new(
            &renderer,
            String::from_utf8_lossy(&data),
            ShaderOptions::mesh_defaults(),
        ))
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
    Texture2D,
    Sampler,
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
                BindGroupOptions {
                    id: BindGroupId::DiffuseTexture,
                    uniforms: vec![
                        UniformOptions {
                            id: UniformId::Texture2D,
                            stage: wgpu::ShaderStages::FRAGMENT,
                            uniform_type: UniformType::Texture2D,
                        },
                        UniformOptions {
                            id: UniformId::Sampler,
                            stage: wgpu::ShaderStages::FRAGMENT,
                            uniform_type: UniformType::Sampler,
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
