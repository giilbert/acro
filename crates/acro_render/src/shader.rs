use std::{borrow::Cow, collections::HashMap, ops::Deref, sync::Arc};

use wgpu::include_wgsl;

use crate::state::RendererHandle;

#[derive(Debug)]
pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new(renderer: &RendererHandle, source: impl ToString) -> Self {
        let module = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(source.to_string())),
            });

        Self { module }
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
