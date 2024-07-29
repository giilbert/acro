use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct RendererHandle {
    state: Arc<RendererState>,
}

#[derive(Debug)]
pub struct RendererState {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    pub(crate) window: Arc<winit::window::Window>,
    pub(crate) encoder: Mutex<Option<wgpu::CommandEncoder>>,
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

        println!("Adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
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
                surface,
                device,
                queue,
                config,
                size,
                window,
                encoder: Mutex::new(None),
            }),
        }
    }

    pub fn clear(&self) {

        // submit will accept anything that implements IntoIter
        // self.queue.submit(std::iter::once(encoder.finish()));
        // output.present();
    }
}

impl Deref for RendererHandle {
    type Target = RendererState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
