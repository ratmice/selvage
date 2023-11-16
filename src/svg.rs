use crate::whisperer::ShapeOpRef;
use kurbo::{Affine, Shape, Size, Stroke};
use peniko::BrushRef;
use peniko::Color;
use std::{fmt, io};
use svg::Node;

//
// FIXME doesn't support clips which is handled fairly differently between
// the original piet which this code comes from and vello.
//

#[derive(Debug, Copy, Clone)]
struct Id(u64);

impl Id {
    // TODO allowing clippy warning temporarily. But this should be changed to impl Display
    #[allow(clippy::inherent_to_string)]
    fn to_string(self) -> String {
        const ALPHABET: &[u8; 52] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut out = String::with_capacity(4);
        let mut x = self.0;
        loop {
            let digit = (x % ALPHABET.len() as u64) as usize;
            out.push(ALPHABET[digit] as char);
            x /= ALPHABET.len() as u64;
            if x == 0 {
                break;
            }
        }
        out
    }
}

impl From<Id> for svg::node::Value {
    fn from(x: Id) -> Self {
        x.to_string().into()
    }
}

use crate::whisperer::SceneBuilderWhisperer;

#[derive(Debug, Clone, Default)]
struct State {
    clip: Option<Id>,
}

pub struct Selvage {
    size: Size,
    // I believe these will still be needed when `PushLayer` is implemented, but since I haven't
    // needed it, I haven't implemented it yet.
    _stack: Vec<State>,
    _state: State,
    doc: svg::Document,
    _next_id: u64,
}

/// An SVG brush
#[derive(Debug, Clone)]
pub struct Brush {
    kind: BrushKind,
}

#[derive(Debug, Clone)]
enum BrushKind {
    Solid(Color),
    Ref(Id),
}

#[derive(Default)]
struct Attrs<'a> {
    xf: Affine,
    clip: Option<Id>,
    fill: Option<(Brush, Option<&'a str>)>,
    stroke: Option<(Brush, &'a Stroke)>,
}

impl Selvage {
    pub fn new(size: Size) -> Self {
        Selvage {
            size,
            doc: svg::Document::new(),
            _next_id: 0,
            _state: State::default(),
            _stack: Vec::new(),
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn write(&self, writer: impl io::Write) -> io::Result<()> {
        svg::write(writer, &self.doc)
    }

    /// Returns an object that can write the svg somewhere.
    pub fn display(&self) -> &impl fmt::Display {
        &self.doc
    }

    // FIXME we don't support clip yet.
    /*
        fn next_id(&mut self) -> Id {
            let x = Id(self.next_id);
            self.next_id += 1;
            x
        }
    */
}

fn xf_val(xf: &Affine) -> svg::node::Value {
    let xf = xf.as_coeffs();
    format!(
        "matrix({} {} {} {} {} {})",
        xf[0], xf[1], xf[2], xf[3], xf[4], xf[5]
    )
    .into()
}

impl Attrs<'_> {
    // allow clippy warning for `width != 1.0` in if statement
    #[allow(clippy::float_cmp)]
    fn apply_to(&self, node: &mut impl Node) {
        node.assign("transform", xf_val(&self.xf));
        if let Some(id) = self.clip {
            node.assign("clip-path", format!("url(#{})", id.to_string()));
        }
        if let Some((ref brush, rule)) = self.fill {
            node.assign("fill", brush.color());
            if let Some(opacity) = brush.opacity() {
                node.assign("fill-opacity", opacity);
            }
            if let Some(rule) = rule {
                node.assign("fill-rule", rule);
            }
        } else {
            node.assign("fill", "none");
        }
        if let Some((ref stroke, style)) = self.stroke {
            node.assign("stroke", stroke.color());
            if let Some(opacity) = stroke.opacity() {
                node.assign("stroke-opacity", opacity);
            }

            if style.width != 1.0 {
                node.assign("stroke-width", style.width);
            }
            use kurbo::Join;
            match style.join {
                Join::Miter => {
                    node.assign("stroke-miterlimit", style.miter_limit);
                }
                Join::Round => {
                    node.assign("stroke-linejoin", "round");
                }
                Join::Bevel => {
                    node.assign("stroke-linejoin", "bevel");
                }
            }
            use kurbo::Cap;
            match style.start_cap {
                Cap::Round => {
                    node.assign("stroke-linecap", "round");
                }
                Cap::Square => {
                    node.assign("stroke-linecap", "square");
                }
                Cap::Butt => (),
            }

            if !style.dash_pattern.is_empty() {
                node.assign("stroke-dasharray", style.dash_pattern.to_vec());
            }
            if style.dash_offset != 0.0 {
                node.assign("stroke-dashoffset", style.dash_offset);
            }
        }
    }
}

impl Brush {
    fn color(&self) -> svg::node::Value {
        match self.kind {
            BrushKind::Solid(color) => fmt_color(color).into(),
            BrushKind::Ref(id) => format!("url(#{})", id.to_string()).into(),
        }
    }

    fn opacity(&self) -> Option<svg::node::Value> {
        match self.kind {
            BrushKind::Solid(color) => Some(fmt_opacity(color).into()),
            BrushKind::Ref(_) => None,
        }
    }
}

/*
impl IntoBrush<Selvage> for Brush {
    fn make_brush<'b>(
        &'b self,
        _piet: &mut Selvage,
        _bbox: impl FnOnce() -> Rect,
    ) -> Cow<'b, Brush> {
        Cow::Owned(self.clone())
    }
}
*/

// RGB in hex representation
fn fmt_color(color: Color) -> String {
    let color = u32::from_ne_bytes(if cfg!(target_endian = "big") {
        [color.r, color.g, color.b, color.a]
    } else {
        [color.a, color.b, color.g, color.r]
    }) >> 8;
    format!("#{:06x}", color)
}

// Opacity as value from [0, 1]
fn fmt_opacity(color: Color) -> String {
    format!("{}", color.a / u8::MAX)
}

fn add_shape(node: &mut impl Node, shape: impl Shape, attrs: &Attrs) {
    if let Some(circle) = shape.as_circle() {
        let mut x = svg::node::element::Circle::new()
            .set("cx", circle.center.x)
            .set("cy", circle.center.y)
            .set("r", circle.radius);
        attrs.apply_to(&mut x);
        node.append(x);
    } else if let Some(round_rect) = shape
        .as_rounded_rect()
        .filter(|r| r.radii().as_single_radius().is_some())
    {
        let mut x = svg::node::element::Rectangle::new()
            .set("x", round_rect.origin().x)
            .set("y", round_rect.origin().y)
            .set("width", round_rect.width())
            .set("height", round_rect.height())
            .set("rx", round_rect.radii().as_single_radius().unwrap())
            .set("ry", round_rect.radii().as_single_radius().unwrap());
        attrs.apply_to(&mut x);
        node.append(x);
    } else if let Some(rect) = shape.as_rect() {
        let mut x = svg::node::element::Rectangle::new()
            .set("x", rect.origin().x)
            .set("y", rect.origin().y)
            .set("width", rect.width())
            .set("height", rect.height());
        attrs.apply_to(&mut x);
        node.append(x);
    } else {
        let mut path = svg::node::element::Path::new().set("d", shape.into_path(1e-3).to_svg());
        attrs.apply_to(&mut path);
        node.append(path);
    }
}

impl SceneBuilderWhisperer for Selvage {
    fn paint_shape_op<'a, 'b>(
        &mut self,
        op: ShapeOpRef<'a, 'b>,
        transform: Affine,
        _brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        match op {
            ShapeOpRef::Fill { brush, .. } => {
                //let foo = brush.to_owned();
                let fill_brush = match brush {
                    BrushRef::Solid(color) => Some((
                        Brush {
                            kind: BrushKind::Solid(color),
                        },
                        None,
                    )),
                    // FIXME
                    _ => None,
                };
                //let brush = brush.make_brush(self, || shape.bounding_box());
                add_shape(
                    &mut self.doc,
                    shape,
                    &Attrs {
                        xf: transform,
                        clip: self._state.clip,
                        fill: fill_brush, // Some((brush, None)),
                        ..Attrs::default()
                    },
                );
            }

            ShapeOpRef::Stroke { style, brush, .. } => {
                let stroke_brush = match brush {
                    BrushRef::Solid(color) => Some((
                        Brush {
                            kind: BrushKind::Solid(color),
                        },
                        style,
                    )),
                    // FIXME
                    _ => None,
                };
                //let brush = brush.make_brush(self, || shape.bounding_box());
                add_shape(
                    &mut self.doc,
                    shape,
                    &Attrs {
                        xf: transform,
                        clip: self._state.clip,
                        stroke: stroke_brush,
                        ..Attrs::default()
                    },
                );
            }
            ShapeOpRef::PushLayer { .. } => {}
        }
    }

    fn paint_shape_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) where
        I: IntoIterator<Item = ShapeOpRef<'a, 'b>>,
    {
        // FIXME this should be able to produce nicer output than just producing an element for
        // each shape_op.
        for op in ops {
            self.paint_shape_op(op, transform, brush_transform, shape)
        }
    }
}
