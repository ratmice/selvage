use kurbo::Rect;
use kurbo::Shape;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug, Clone)]
pub enum StaticShape {
    PathSeg(kurbo::PathSeg),
    Arc(kurbo::Arc),
    BezPath(kurbo::BezPath),
    Circle(kurbo::Circle),
    CircleSegment(kurbo::CircleSegment),
    CubicBez(kurbo::CubicBez),
    Ellipse(kurbo::Ellipse),
    Line(kurbo::Line),
    QuadBez(kurbo::QuadBez),
    Rect(kurbo::Rect),
    RoundedRect(kurbo::RoundedRect),
}
macro_rules! from_shape {
    ($it: ident) => {
        impl From<kurbo::$it> for StaticShape {
            fn from(it: kurbo::$it) -> Self {
                Self::$it(it)
            }
        }
    };
}

from_shape!(PathSeg);
from_shape!(Arc);
from_shape!(BezPath);
from_shape!(Circle);
from_shape!(CircleSegment);
from_shape!(CubicBez);
from_shape!(Ellipse);
from_shape!(Line);
from_shape!(QuadBez);
from_shape!(Rect);
from_shape!(RoundedRect);

impl Shape for StaticShape {
    type PathElementsIter<'iter> = Box<dyn Iterator<Item = kurbo::PathEl> + 'iter>;
    fn path_elements(&self, tol: f64) -> Box<dyn Iterator<Item = kurbo::PathEl> + '_> {
        use StaticShape as S;
        match self {
            S::PathSeg(it) => Box::new(it.path_elements(tol)),
            S::Arc(it) => Box::new(it.path_elements(tol)),
            S::BezPath(it) => Box::new(it.path_elements(tol)),
            S::Circle(it) => Box::new(it.path_elements(tol)),
            S::CircleSegment(it) => Box::new(it.path_elements(tol)),
            S::CubicBez(it) => Box::new(it.path_elements(tol)),
            S::Ellipse(it) => Box::new(it.path_elements(tol)),
            S::Line(it) => Box::new(it.path_elements(tol)),
            S::QuadBez(it) => Box::new(it.path_elements(tol)),
            S::Rect(it) => Box::new(it.path_elements(tol)),
            S::RoundedRect(it) => Box::new(it.path_elements(tol)),
        }
    }

    fn perimeter(&self, acc: f64) -> f64 {
        use StaticShape as S;
        match self {
            S::PathSeg(it) => it.perimeter(acc),
            S::Arc(it) => it.perimeter(acc),
            S::BezPath(it) => it.perimeter(acc),
            S::Circle(it) => it.perimeter(acc),
            S::CircleSegment(it) => it.perimeter(acc),
            S::CubicBez(it) => it.perimeter(acc),
            S::Ellipse(it) => it.perimeter(acc),
            S::Line(it) => it.perimeter(acc),
            S::QuadBez(it) => it.perimeter(acc),
            S::Rect(it) => it.perimeter(acc),
            S::RoundedRect(it) => it.perimeter(acc),
        }
    }

    fn area(&self) -> f64 {
        use StaticShape as S;
        match self {
            S::PathSeg(it) => it.area(),
            S::Arc(it) => it.area(),
            S::BezPath(it) => it.area(),
            S::Circle(it) => it.area(),

            S::CircleSegment(it) => it.area(),
            S::CubicBez(it) => it.area(),
            S::Ellipse(it) => it.area(),
            S::Line(it) => it.area(),
            S::QuadBez(it) => it.area(),
            S::Rect(it) => it.area(),
            S::RoundedRect(it) => it.area(),
        }
    }

    fn winding(&self, pt: kurbo::Point) -> i32 {
        use StaticShape as S;
        match self {
            S::PathSeg(it) => it.winding(pt),
            S::Arc(it) => it.winding(pt),
            S::BezPath(it) => it.winding(pt),
            S::Circle(it) => it.winding(pt),

            S::CircleSegment(it) => it.winding(pt),
            S::CubicBez(it) => it.winding(pt),
            S::Ellipse(it) => it.winding(pt),
            S::Line(it) => it.winding(pt),
            S::QuadBez(it) => it.winding(pt),
            S::Rect(it) => it.winding(pt),
            S::RoundedRect(it) => it.winding(pt),
        }
    }

    fn bounding_box(&self) -> Rect {
        use StaticShape as S;
        match self {
            S::PathSeg(it) => it.bounding_box(),
            S::Arc(it) => it.bounding_box(),
            S::BezPath(it) => it.bounding_box(),
            S::Circle(it) => it.bounding_box(),

            S::CircleSegment(it) => it.bounding_box(),
            S::CubicBez(it) => it.bounding_box(),
            S::Ellipse(it) => it.bounding_box(),
            S::Line(it) => it.bounding_box(),
            S::QuadBez(it) => it.bounding_box(),
            S::Rect(it) => it.bounding_box(),
            S::RoundedRect(it) => it.bounding_box(),
        }
    }
}
