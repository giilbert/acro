use acro_assets::Loadable;
use acro_ecs::World;
use image::GenericImageView;
use tracing::info;

use crate::state::RendererHandle;

#[derive(Debug)]
pub struct Texture {
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
}

impl Loadable for Texture {
    fn load(world: &World, data: Vec<u8>) -> Result<Self, ()> {
        // TODO: error handling
        let image = image::load_from_memory(&data).map_err(|_| ())?;
        let image_rgba = image.to_rgba8();

        let dimensions = image.dimensions();

        let renderer = world.resources().get::<RendererHandle>();

        info!("Loaded image with dimensions {:?}", dimensions);
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Texture {
            texture_view,
            sampler,
        })
    }
}
