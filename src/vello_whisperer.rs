#![cfg(feature = "vello")]
use crate::whisperer::*;
use kurbo::{Affine, Shape};
use vello::SceneBuilder;

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
