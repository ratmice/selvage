use serde::{Deserialize, Serialize};
mod whisperer;
pub use crate::whisperer::{SceneBuilderWhisperer, ShapeOp, ShapeOpRef};
mod shape;
pub use shape::StaticShape;
mod svg;
pub use svg::Selvage;
mod shape_transform;
pub use shape_transform::ShapeTransform;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Failed in serde_json")]
    SerdeJson(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
