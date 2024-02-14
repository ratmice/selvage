use crate::shape::StaticShape;
use crate::whisperer::PaintOpRef;
use crate::SceneWhisperer;
use kurbo::{Affine, Shape};
pub struct ShapeTransform<'a, T: SceneWhisperer> {
    whisperer: &'a mut T,
    tolerance: f64,
}

// To avoid applying an affine transform to a `Stroke{width,..}`
// This applies transforms to the shape first, then gives the renderer an AFFINE::IDENTITY.
// Unlike `SceneWhisperer` which takes a &impl Shape, this takes an owned `StaticShape`,
// It should probably take a `&StaticShape`, however `Mul` isn't implemented for these as references.
impl<'w, T: SceneWhisperer> ShapeTransform<'w, T> {
    pub fn new(whisperer: &'w mut T, tolerance: f64) -> Self {
        Self {
            whisperer,
            tolerance,
        }
    }

    /// Applies the transform to the shape, then paints with Affine::IDENTITY
    pub fn transform_paint_op(
        &mut self,
        op: PaintOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: StaticShape,
    ) {
        self.whisperer.apply_paint_op(
            op,
            Affine::IDENTITY,
            brush_transform,
            &shape.apply_transform(transform, self.tolerance),
        );
    }

    /// Applies the transform to the shape, then paints with Affine::IDENTITY
    pub fn transform_paint_shape_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: StaticShape,
    ) where
        I: IntoIterator<Item = PaintOpRef<'a, 'b>>,
    {
        self.whisperer.apply_paint_ops(
            ops,
            Affine::IDENTITY,
            brush_transform,
            &shape.apply_transform(transform, self.tolerance),
        );
    }
}

impl<'w, T: SceneWhisperer> SceneWhisperer for ShapeTransform<'w, T> {
    /// Calls paint_shape_op on `self.whisperer` directly,
    /// without flattening the transform on shape first.
    fn apply_paint_op(
        &mut self,
        op: PaintOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.whisperer
            .apply_paint_op(op, transform, brush_transform, shape);
    }

    /// Calls `paint_shape_ops` on `self.whisperer` directly,
    /// without flattening the transform on shape first.
    fn apply_paint_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) where
        I: IntoIterator<Item = PaintOpRef<'a, 'b>>,
    {
        self.whisperer
            .apply_paint_ops(ops, transform, brush_transform, shape);
    }
}
