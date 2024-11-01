use anyhow::{anyhow, bail, Result};
use kurbo::Size;
use peniko::Color;
#[cfg(feature = "vello")]
use vello::{
    util::block_on_wgpu, peniko, util::RenderContext, AaConfig, AaSupport, RendererOptions, Scene,
};
use wgpu::BufferUsages;
use wgpu::ImageCopyBuffer;
use wgpu::{
    BufferDescriptor, CommandEncoderDescriptor, Extent3d, TextureDescriptor, TextureFormat,
    TextureUsages,
};

#[cfg(feature = "vello")]
pub struct Renderer {
    device_id: usize,
    extents: Extent3d,
    ctxt: RenderContext,
    renderer: vello::Renderer,
}
#[cfg(feature = "vello")]
impl Renderer {
    pub async fn new(sz: Size) -> anyhow::Result<Self> {
        let width = sz.width.ceil() as u32;
        let height = sz.height.ceil() as u32;
        let extents = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let mut ctxt = RenderContext::new();

        let device_id = ctxt
            .device(None)
            .await
            .ok_or_else(|| anyhow!("No compatible device found"))?;
        let device_handle = &mut ctxt.devices[device_id];
        let device = &device_handle.device;
        let renderer = vello::Renderer::new(
            device,
            RendererOptions {
                surface_format: None,
                num_init_threads: None,
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
            },
        )
        .or_else(|_| bail!("Got non-Send/Sync error from creating renderer"))?;
        Ok(Renderer {
            device_id,
            extents,
            ctxt,
            renderer,
        })
    }

    pub fn texture(&mut self, name: &str) -> wgpu::Texture {
        let device_handle = &mut self.ctxt.devices[self.device_id];
        let device = &device_handle.device;
        device.create_texture(&TextureDescriptor {
            label: Some(name),
            size: self.extents,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    pub async fn render<P: AsRef<std::path::Path>>(
        &mut self,
        scene: Scene,
        out_path: P,
        target: &mut wgpu::Texture,
        background: Color,
    ) -> Result<()> {
        let size = self.extents;
        let device_handle = &mut self.ctxt.devices[self.device_id];
        let device = &device_handle.device;
        let queue = &device_handle.queue;
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let render_params = vello::RenderParams {
            base_color: background,
            width: self.extents.width,
            height: self.extents.height,
            antialiasing_method: AaConfig::Msaa16,
        };
        self.renderer
            .render_to_texture(device, queue, &scene, &view, &render_params)
            .or_else(|_| bail!("Got non-send/sync error from rendering"))?;
        let padded_byte_width = {
            let w = self.extents.width * 4;
            match w % 256 {
                0 => w,
                r => w + (256 - r),
            }
        };
        let buffer_size = padded_byte_width as u64 * self.extents.height as u64;
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("val"),
            size: buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Copy out buffer"),
        });
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_byte_width),
                    rows_per_image: None,
                },
            },
            size,
        );
        queue.submit([encoder.finish()]);
        let buf_slice = buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buf_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        if let Some(recv_result) = block_on_wgpu(device, receiver.receive()) {
            recv_result?;
        } else {
            bail!("channel was closed");
        }

        let data = buf_slice.get_mapped_range();
        let mut result_unpadded =
            Vec::<u8>::with_capacity((self.extents.width * self.extents.height * 4).try_into()?);
        for row in 0..self.extents.height {
            let start = (row * padded_byte_width).try_into()?;
            result_unpadded.extend(&data[start..start + (self.extents.width * 4) as usize]);
        }

        let mut file = std::fs::File::create(out_path)?;
        let mut encoder = png::Encoder::new(&mut file, self.extents.width, self.extents.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&result_unpadded)?;
        writer.finish()?;
        Ok(())
    }
}
