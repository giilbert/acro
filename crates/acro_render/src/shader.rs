use std::{borrow::Cow, collections::HashMap, ops::Deref, sync::Arc};

use acro_assets::{Asset, Assets, Loadable, LoaderContext};
use acro_ecs::World;
use acro_math::{Float, Mat4};

use crate::{state::RendererHandle, Texture};

#[derive(Debug)]
pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) bind_groups: HashMap<BindGroupId, BindGroupData>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Deserialize)]
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

#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Deserialize)]
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
    pub(crate) data_type: UniformDataType,
}

#[derive(Debug)]
pub enum UniformDataType {
    Buffer(wgpu::Buffer),
    Texture(Asset<Texture>),
    Sampler(Asset<Texture>),
}

impl Shader {
    pub fn new(
        renderer: &RendererHandle,
        diffuse_texture: Asset<Texture>,
        source: impl ToString,
        options: Arc<ShaderOptions>,
    ) -> Self {
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
                    UniformType::Sampler => {
                        uniform_data.insert(
                            uniform_options.id.clone(),
                            UniformData {
                                index: index as u32,
                                stage: uniform_options.stage,
                                data_type: UniformDataType::Sampler(diffuse_texture.clone()),
                            },
                        );
                        continue;
                    }
                    UniformType::Texture2D => {
                        uniform_data.insert(
                            uniform_options.id.clone(),
                            UniformData {
                                index: index as u32,
                                stage: uniform_options.stage,
                                data_type: UniformDataType::Texture(diffuse_texture.clone()),
                            },
                        );
                        continue;
                    }
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
                        data_type: UniformDataType::Buffer(buffer),
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
                        .map(|data| match &data.data_type {
                            UniformDataType::Buffer(buffer) => wgpu::BindGroupEntry {
                                binding: data.index,
                                resource: buffer.as_entire_binding(),
                            },
                            UniformDataType::Sampler(_) => wgpu::BindGroupEntry {
                                binding: data.index,
                                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                            },
                            UniformDataType::Texture(_) => wgpu::BindGroupEntry {
                                binding: data.index,
                                resource: wgpu::BindingResource::TextureView(
                                    &diffuse_texture.texture_view,
                                ),
                            },
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
    type Config = ShaderOptions;

    fn load(ctx: &LoaderContext, config: Arc<Self::Config>, data: Vec<u8>) -> Result<Self, ()> {
        // TODO: Add asset dependencies (a foolproof way to ensure all assets are loaded in order)
        let texture = ctx.load_dependent(ctx, &config.diffuse_texture);

        let renderer = ctx
            .system_run_context
            .world
            .resources()
            .get::<RendererHandle>();

        Ok(Shader::new(
            &renderer,
            texture,
            String::from_utf8_lossy(&data),
            config,
            // ShaderOptions::mesh_defaults(),
        ))
    }
}

impl UniformDataType {
    pub fn assert_buffer(&self) -> &wgpu::Buffer {
        match self {
            Self::Buffer(buffer) => buffer,
            _ => panic!("Expected buffer, found texture or sampler"),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ShaderOptions {
    pub diffuse_texture: String,
    pub bind_groups: Vec<BindGroupOptions>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BindGroupOptions {
    pub(crate) id: BindGroupId,
    pub(crate) uniforms: Vec<UniformOptions>,
}

#[derive(Debug, serde::Deserialize)]
pub enum UniformType {
    Mat4,
    Texture2D,
    Sampler,
}

#[derive(Debug, serde::Deserialize)]
pub struct UniformOptions {
    pub(crate) id: UniformId,
    pub(crate) stage: wgpu::ShaderStages,
    pub(crate) uniform_type: UniformType,
}
