use std::sync::Arc;

use acro_assets::{Loadable, LoaderContext};
use acro_ecs::World;
use image::GenericImageView;
use tracing::info;

use crate::state::RendererHandle;

#[derive(Debug)]
pub struct Texture {
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for Texture {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for Texture {}

#[derive(Debug, serde::Deserialize)]
pub struct TextureOptions {
    address_mode_u: wgpu::AddressMode,
    address_mode_v: wgpu::AddressMode,
    address_mode_w: wgpu::AddressMode,
    mag_filter: wgpu::FilterMode,
    min_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::FilterMode,
}

impl Loadable for Texture {
    type Config = TextureOptions;

    fn load(ctx: &LoaderContext, config: Arc<Self::Config>, data: Vec<u8>) -> eyre::Result<Self> {
        let image = image::load_from_memory(&data)?;
        let image_rgba = image.to_rgba8();

        let dimensions = image.dimensions();

        let renderer = ctx
            .system_run_context
            .world
            .resources()
            .get::<RendererHandle>();

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
            address_mode_u: config.address_mode_u,
            address_mode_v: config.address_mode_v,
            address_mode_w: config.address_mode_w,
            mag_filter: config.mag_filter,
            min_filter: config.min_filter,
            mipmap_filter: config.mipmap_filter,
            ..Default::default()
        });

        Ok(Texture {
            texture_view,
            sampler,
        })
    }
}
