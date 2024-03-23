#![allow(unused)]
use kurbo::{Affine, Point, Rect, Shape, Stroke};
use peniko::{BlendMode, Brush, BrushRef, Fill};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

pub enum PaintOpRef<'a, 'b> {
    Fill {
        style: Fill,
        brush: BrushRef<'b>,
    },
    Stroke {
        style: &'a Stroke,
        brush: BrushRef<'b>,
    },
    PushLayer {
        blend: BlendMode,
        alpha: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaintOp {
    Fill { style: Fill, brush: Brush },
    Stroke { style: Stroke, brush: Brush },
    PushLayer { blend: BlendMode, alpha: f32 },
}

impl<'a> From<&'a PaintOp> for PaintOpRef<'a, 'a> {
    fn from(x: &'a PaintOp) -> PaintOpRef<'a, 'a> {
        match x {
            PaintOp::Fill { style, brush } => PaintOpRef::Fill {
                style: *style,
                brush: brush.into(),
            },
            PaintOp::Stroke { style, brush } => PaintOpRef::Stroke {
                style,
                brush: brush.into(),
            },
            PaintOp::PushLayer { blend, alpha } => PaintOpRef::PushLayer {
                blend: *blend,
                alpha: *alpha,
            },
        }
    }
}

pub trait SceneWhisperer {
    fn apply_paint_op(
        &mut self,
        op: PaintOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    );
    fn apply_paint_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) where
        I: IntoIterator<Item = PaintOpRef<'a, 'b>>;
}
