use wgpu::*;
use anyhow::Result;
use image::GenericImageView;

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler
}

impl Texture {
    pub fn from_image(device: &Device, queue: &Queue, img: &image::DynamicImage, label: Option<&str>) -> Result<Self> {
        let rgba = img.to_rgba8();

        let dimensions = img.dimensions();

        let texture_size = Extent3d {
            width: dimensions.0, 
            height: dimensions.1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(
            &TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                #[cfg(target_arch="wasm32")]
                format: TextureFormat::Rgba8Unorm,
                #[cfg(not(target_arch="wasm32"))]
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                label,
                view_formats: &[]
            }
        );

        queue.write_texture(
           TexelCopyTextureInfo { 
                texture: &texture, 
                mip_level: 0, 
                origin: Origin3d::ZERO, 
                aspect: TextureAspect::All 
            }, 
            &rgba,
            TexelCopyBufferLayout { 
                offset: 0, 
                bytes_per_row: Some(4 * dimensions.0), 
                rows_per_image: Some(dimensions.1)
            }, 
            texture_size
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &SamplerDescriptor { 
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                ..Default::default()
            }
        );

        Ok(Self {
            sampler,
            texture,
            view: texture_view
        })
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8], 
        label: &str
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }
}