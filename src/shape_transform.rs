use crate::shape::StaticShape;
use crate::whisperer::ShapeOpRef;
use crate::SceneBuilderWhisperer;
use kurbo::{Affine, Shape};
pub struct ShapeTransform<'a, T: SceneBuilderWhisperer> {
    whisperer: &'a mut T,
    tolerance: f64,
}

// To avoid applying an affine transform to a `Stroke{width,..}`
// This applies transforms to the shape first, then gives the renderer an AFFINE::IDENTITY.
// Unlike `SceneBuilderWhisperer` which takes a &impl Shape, this takes an owned `StaticShape`,
// It should probably take a `&StaticShape`, however `Mul` isn't implemented for these as references.
impl<'w, T: SceneBuilderWhisperer> ShapeTransform<'w, T> {
    pub fn new(whisperer: &'w mut T, tolerance: f64) -> Self {
        Self {
            whisperer,
            tolerance,
        }
    }

    /// Applies the transform to the shape, then paints with Affine::IDENTITY
    pub fn transform_paint_shape_op(
        &mut self,
        op: ShapeOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: StaticShape,
    ) {
        self.whisperer.paint_shape_op(
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
        I: IntoIterator<Item = ShapeOpRef<'a, 'b>>,
    {
        self.whisperer.paint_shape_ops(
            ops,
            Affine::IDENTITY,
            brush_transform,
            &shape.apply_transform(transform, self.tolerance),
        );
    }
}

impl<'w, T: SceneBuilderWhisperer> SceneBuilderWhisperer for ShapeTransform<'w, T> {
    /// Calls paint_shape_op on `self.whisperer` directly,
    /// without flattening the transform on shape first.
    fn paint_shape_op(
        &mut self,
        op: ShapeOpRef<'_, '_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.whisperer
            .paint_shape_op(op, transform, brush_transform, shape);
    }

    /// Calls `paint_shape_ops` on `self.whisperer` directly,
    /// without flattening the transform on shape first.
    fn paint_shape_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) where
        I: IntoIterator<Item = ShapeOpRef<'a, 'b>>,
    {
        self.whisperer
            .paint_shape_ops(ops, transform, brush_transform, shape);
    }
}
