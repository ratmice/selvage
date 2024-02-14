mod whisperer;
pub use crate::whisperer::{PaintOp, PaintOpRef, SceneWhisperer};
mod shape;
pub use shape::StaticShape;
mod shape_transform;
pub use shape_transform::ShapeTransform;

#[cfg(feature = "vello")]
mod vello_whisperer;
#[cfg(feature = "vello")]
pub use vello_whisperer::*;
#[cfg(feature = "svg")]
mod svg;
#[cfg(feature = "svg")]
pub use svg::Svg;

#[cfg(feature = "pdf")]
mod pdf_whisperer;
#[cfg(feature = "pdf")]
pub use pdf_whisperer::*;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
