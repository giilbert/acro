use std::{
    cell::{Ref, RefCell},
    ops::Deref,
    sync::Arc,
};

use tracing::info;
use wgpu::MemoryHints;
use winit::dpi::PhysicalSize;

#[derive(Debug, Clone)]
pub struct RendererHandle {
    state: Arc<RendererState>,
}

#[derive(Debug)]
pub struct RendererState {
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: RefCell<wgpu::SurfaceConfiguration>,
    pub size: RefCell<winit::dpi::PhysicalSize<u32>>,
    pub window: Arc<winit::window::Window>,
    pub frame_state: RefCell<Option<FrameState>>,
}

#[derive(Debug)]
pub struct FrameState {
    pub encoder: RefCell<wgpu::CommandEncoder>,
    pub view: wgpu::TextureView,
    pub output: wgpu::SurfaceTexture,
}

impl RendererState {
    pub async fn new(window: Arc<winit::window::Window>) -> RendererHandle {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed to request adapter");

        info!("Adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: MemoryHints::Performance,
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        RendererHandle {
            state: Arc::new(RendererState {
                adapter,
                surface,
                device,
                queue,
                config: RefCell::new(config),
                size: RefCell::new(size),
                window,
                frame_state: RefCell::new(None),
            }),
        }
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        *self.size.borrow_mut() = size;

        if size.width != 0 && size.height != 0 {
            self.config.borrow_mut().width = size.width;
            self.config.borrow_mut().height = size.height;
            self.surface.configure(&self.device, &self.config.borrow());
        }
    }

    pub fn take_frame_state(&self) -> Option<FrameState> {
        self.frame_state.borrow_mut().take()
    }

    pub fn frame_state(&self) -> Ref<FrameState> {
        Ref::map(self.frame_state.borrow(), |state| state.as_ref().unwrap())
    }
}

impl Deref for RendererHandle {
    type Target = RendererState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
