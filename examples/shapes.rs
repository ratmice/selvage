use kurbo::{Affine, BezPath, Ellipse, Size};
use peniko::Color;
#[cfg(feature = "pdf")]
use selvage::Pdf;
#[cfg(feature = "svg")]
use selvage::Svg;
use selvage::{PaintOp, SceneWhisperer, StaticShape};
#[cfg(feature = "vello")]
use vello::Scene;

#[path = "./utils/render.rs"]
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
    let mut shapes = Vec::new();

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
        paint_ops: vec![stroke.clone()],
        transform: Affine::IDENTITY,
        brush_transform: None,
    };
    shapes.push(sym);
    shapes.push(Symbol {
        shape: StaticShape::Ellipse(Ellipse::new((32.0, 32.0), (16.0, 8.0), 0.0)),
        paint_ops: vec![stroke],
        transform: Affine::IDENTITY,
        brush_transform: None,
    });
    Ok(serde_json::to_string(&shapes)?)
}

fn main() -> anyhow::Result<()> {
    let s = serialized_shape()?;
    let shapes: Vec<Symbol> = serde_json::from_str(&s)?;

    #[cfg(not(any(feature = "svg", feature = "vello", feature = "pdf")))]
    eprintln!("Must enable feature vello, svg, or pdf to do anything");

    #[cfg(feature = "vello")]
    let mut scene = Scene::new();
    #[cfg(feature = "svg")]
    let mut svg = Svg::new(RENDER_SIZE);
    #[cfg(feature = "pdf")]
    let mut pdf = Pdf::new(RENDER_SIZE, 0.1);

    for shape in shapes {
        #[cfg(feature = "vello")]
        {
            scene.apply_paint_ops(
                shape.paint_ops.iter().map(|x| x.into()),
                shape.transform,
                shape.brush_transform,
                &shape.shape,
            );
        }
        #[cfg(feature = "svg")]
        {
            svg.apply_paint_ops(
                shape.paint_ops.iter().map(|x| x.into()),
                shape.transform,
                shape.brush_transform,
                &shape.shape,
            );
        }
        #[cfg(feature = "pdf")]
        {
            pdf.apply_paint_ops(
                shape.paint_ops.iter().map(|x| x.into()),
                shape.transform,
                shape.brush_transform,
                &shape.shape,
            );
        }
    }
    #[cfg(feature = "pdf")]
    {
        let mut path_buf = std::path::PathBuf::from(OUTPUT_NAME);
        path_buf.set_extension("pdf");
        let out = std::fs::File::create(path_buf)?;
        pdf.write(out)?;
    }
    #[cfg(feature = "svg")]
    {
        let mut path_buf = std::path::PathBuf::from(OUTPUT_NAME);
        path_buf.set_extension("svg");
        let svg_out = std::fs::File::create(path_buf)?;
        svg.write(svg_out)?;
    }
    #[cfg(feature = "vello")]
    {
        let mut r = pollster::block_on(Renderer::new(RENDER_SIZE))?;
        let mut target = r.texture("foo");
        let mut path_buf = std::path::PathBuf::from(OUTPUT_NAME);
        path_buf.set_extension("png");
        pollster::block_on(r.render(scene, path_buf, &mut target, BG_COLOR))?;
    }
    Ok(())
}
