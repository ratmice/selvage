#![allow(unused)]
use kurbo::{Affine, Point, Rect, Shape, Stroke};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use vello::peniko::{BlendMode, Brush, BrushRef, Fill};
use vello::SceneBuilder;

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

impl SceneBuilderWhisperer for SceneBuilder<'_> {
    fn paint_shape_op(
        &mut self,
        op: ShapeOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        match op {
            ShapeOpRef::Fill { style, brush } => {
                self.fill(style, transform, brush, brush_transform, shape)
            }
            ShapeOpRef::Stroke { style, brush } => {
                self.stroke(style, transform, brush, brush_transform, shape)
            }
            ShapeOpRef::PushLayer { blend, alpha } => {
                self.push_layer(blend, alpha, transform, shape)
            }
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
        for op in ops {
            self.paint_shape_op(op, transform, brush_transform, shape)
        }
    }
}
