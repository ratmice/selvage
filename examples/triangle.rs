use kurbo::BezPath;
use kurbo::{Affine, Size};
use peniko::Color;
#[cfg(feature = "pdf")]
use selvage::Pdf;
#[cfg(feature = "svg")]
use selvage::Svg;
use selvage::{PaintOp, SceneWhisperer, StaticShape};
#[cfg(feature = "vello")]
use vello::Scene;

#[path = "utils/render.rs"]
mod utils;
#[cfg(feature = "vello")]
use utils::Renderer;

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
        // Static shape isn't really utilized here,
        // see the *shapes.rs* example, for an example
        // where it is necessary
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
            pollster::block_on(renderer.render(scene, path_buf, &mut target, BG_COLOR))?;
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
