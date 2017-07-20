pub mod point;
pub mod line;
pub mod triangle;

use ::stencil::{StencilTest, StencilOp};
use ::mesh::{Vertex, Mesh};
use ::geometry::{Dimensions, Coordinate, FaceWinding};

use ::pipeline::PipelineObject;

use ::pipeline::types::{Pixel, StencilValue};

#[derive(Clone, Copy)]
pub struct RasterArguments<P, V> where P: PipelineObject, V: Vertex {
    pub dimensions: Dimensions,
    pub tile: (Coordinate, Coordinate),
    pub bounds: ((V::Scalar, V::Scalar), (V::Scalar, V::Scalar)),
    pub stencil_value: StencilValue<P>,
    pub stencil_test: StencilTest,
    pub stencil_op: StencilOp,
    pub antialiased_lines: bool,
    pub cull_faces: Option<FaceWinding>,
}

pub use self::triangle::rasterize_triangle;
pub use self::line::rasterize_line;
pub use self::point::rasterize_point;