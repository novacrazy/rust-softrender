//! Shader geometry structures

pub mod dimension;
pub mod coordinate;
pub mod winding;
pub mod clipvertex;
pub mod screenvertex;

pub use self::dimension::{Dimensions, HasDimensions};
pub use self::coordinate::Coordinate;
pub use self::winding::FaceWinding;
pub use self::clipvertex::ClipVertex;
pub use self::screenvertex::ScreenVertex;