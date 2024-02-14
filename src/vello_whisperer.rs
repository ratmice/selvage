#![cfg(feature = "vello")]
use crate::whisperer::*;
use kurbo::{Affine, Shape};
use vello::Scene;

impl SceneWhisperer for Scene {
    fn apply_paint_op(
        &mut self,
        op: PaintOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        match op {
            PaintOpRef::Fill { style, brush } => {
                self.fill(style, transform, brush, brush_transform, shape)
            }
            PaintOpRef::Stroke { style, brush } => {
                self.stroke(style, transform, brush, brush_transform, shape)
            }
            PaintOpRef::PushLayer { blend, alpha } => {
                self.push_layer(blend, alpha, transform, shape)
            }
        }
    }
    fn apply_paint_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) where
        I: IntoIterator<Item = PaintOpRef<'a, 'b>>,
    {
        for op in ops {
            self.apply_paint_op(op, transform, brush_transform, shape)
        }
    }
}
