#![allow(unused)]
use kurbo::{Affine, Point, Rect, Shape, Stroke};
use peniko::{BlendMode, Brush, BrushRef, Fill};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

#[derive(Clone, Debug, Serialize)]
pub enum ShapeOpRef<'a, 'b> {
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
pub enum ShapeOp {
    Fill { style: Fill, brush: Brush },
    Stroke { style: Stroke, brush: Brush },
    PushLayer { blend: BlendMode, alpha: f32 },
}

impl<'a> From<&'a ShapeOp> for ShapeOpRef<'a, 'a> {
    fn from(x: &'a ShapeOp) -> ShapeOpRef<'a, 'a> {
        match x {
            ShapeOp::Fill { style, brush } => ShapeOpRef::Fill {
                style: *style,
                brush: brush.into(),
            },
            ShapeOp::Stroke { style, brush } => ShapeOpRef::Stroke {
                style,
                brush: brush.into(),
            },
            ShapeOp::PushLayer { blend, alpha } => ShapeOpRef::PushLayer {
                blend: *blend,
                alpha: *alpha,
            },
        }
    }
}

pub trait SceneBuilderWhisperer {
    fn paint_shape_op(
        &mut self,
        op: ShapeOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    );
    fn paint_shape_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) where
        I: IntoIterator<Item = ShapeOpRef<'a, 'b>>;
}
