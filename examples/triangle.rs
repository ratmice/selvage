use anyhow::{anyhow, bail, Result};
use kurbo::BezPath;
use kurbo::{Affine, Size};
use peniko::Color;
#[cfg(feature = "pdf")]
use selvage::Pdf;
#[cfg(feature = "svg")]
use selvage::Svg;
use selvage::{PaintOp, SceneWhisperer, StaticShape};
#[cfg(feature = "vello")]
use vello::{
    block_on_wgpu, peniko, util::RenderContext, AaConfig, AaSupport, RendererOptions, Scene,
};
use wgpu::BufferUsages;
use wgpu::ImageCopyBuffer;
use wgpu::{
    BufferDescriptor, CommandEncoderDescriptor, Extent3d, TextureDescriptor, TextureFormat,
    TextureUsages,
};

const RENDER_SIZE: Size = Size {
    width: 64.0,
    height: 64.0,
};
const BG_COLOR: Color = peniko::Color::LIGHT_GRAY;
const PEN_COLOR: Color = peniko::Color::BLACK;
const STROKE_WIDTH: f64 = 1.0;
const OUTPUT_NAME: &str = "out";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub shape: StaticShape,
    pub paint_ops: Vec<PaintOp>,
    pub transform: kurbo::Affine,
    pub brush_transform: Option<kurbo::Affine>,
}

fn serialized_shape() -> anyhow::Result<String> {
    let mut tri = BezPath::new();
    tri.move_to((10., 10.));
    tri.line_to((15., 15.));
    tri.line_to((10.0, 20.0));
    tri.close_path();

    let stroke = PaintOp::Stroke {
        style: kurbo::Stroke::new(STROKE_WIDTH).with_caps(kurbo::Cap::Butt),
        brush: peniko::Brush::Solid(PEN_COLOR),
    };
    let sym = Symbol {
        shape: StaticShape::BezPath(tri),
        paint_ops: vec![stroke],
        transform: Affine::IDENTITY,
        brush_transform: None,
    };
    Ok(serde_json::to_string(&sym)?)
}

fn main() -> anyhow::Result<()> {
    let s = serialized_shape()?;
    let sym: Symbol = serde_json::from_str(&s)?;

    #[cfg(not(any(feature = "svg", feature = "vello", feature = "pdf")))]
    eprintln!("Must enable feature vello, svg, or pdf to do anything");

    {
        #[cfg(feature = "vello")]
        {
            let mut renderer = pollster::block_on(Renderer::new(RENDER_SIZE))?;
            let mut target = renderer.texture("foo");
            let mut scene = Scene::new();
            scene.apply_paint_ops(
                sym.paint_ops.iter().map(|x| x.into()),
                sym.transform,
                sym.brush_transform,
                &sym.shape,
            );
            let mut path_buf = std::path::PathBuf::from(OUTPUT_NAME);
            path_buf.set_extension("png");
            pollster::block_on(renderer.render(scene, path_buf, &mut target))?;
        }
        #[cfg(feature = "svg")]
        {
            let mut svg = Svg::new(RENDER_SIZE);
            let mut path_buf = std::path::PathBuf::from(OUTPUT_NAME);
            path_buf.set_extension("svg");
            svg.apply_paint_ops(
                sym.paint_ops.iter().map(|x| x.into()),
                sym.transform,
                sym.brush_transform,
                &sym.shape,
            );

            let svg_out = std::fs::File::create(path_buf)?;
            svg.write(svg_out)?;
        }
        #[cfg(feature = "pdf")]
        {
            let mut pdf = Pdf::new(RENDER_SIZE, 0.1);
            let mut path_buf = std::path::PathBuf::from(OUTPUT_NAME);
            pdf.apply_paint_ops(
                sym.paint_ops.iter().map(|x| x.into()),
                sym.transform,
                sym.brush_transform,
                &sym.shape,
            );
            path_buf.set_extension("pdf");
            let out = std::fs::File::create(path_buf)?;
            pdf.write(out)?;
        }
    }
    Ok(())
}

#[cfg(feature = "vello")]
struct Renderer {
    device_id: usize,
    extents: Extent3d,
    ctxt: RenderContext,
    renderer: vello::Renderer,
}
#[cfg(feature = "vello")]
impl Renderer {
    async fn new(sz: Size) -> anyhow::Result<Self> {
        let width = sz.width.ceil() as u32;
        let height = sz.height.ceil() as u32;
        let extents = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let mut ctxt = RenderContext::new()
            .or_else(|_| bail!("got non-Send/Sync error from creating render context"))?;

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

    fn texture(&mut self, name: &str) -> wgpu::Texture {
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

    async fn render<P: AsRef<std::path::Path>>(
        &mut self,
        scene: Scene,
        out_path: P,
        target: &mut wgpu::Texture,
    ) -> Result<()> {
        let size = self.extents;
        let device_handle = &mut self.ctxt.devices[self.device_id];
        let device = &device_handle.device;
        let queue = &device_handle.queue;
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let render_params = vello::RenderParams {
            base_color: BG_COLOR,
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
