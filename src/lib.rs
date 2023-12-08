mod whisperer;
pub use crate::whisperer::{SceneBuilderWhisperer, ShapeOp, ShapeOpRef};
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
pub use svg::Selvage;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
