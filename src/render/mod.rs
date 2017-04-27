//! Rendering pipeline implementation

pub mod geometry;
pub mod framebuffer;
pub mod uniform;
pub mod pipeline;

pub use self::geometry::{FaceWinding, ClipVertex, ScreenVertex};
pub use self::framebuffer::FrameBuffer;
pub use self::uniform::{BarycentricInterpolation, barycentric_interpolate};
pub use self::pipeline::{Pipeline, VertexShader, FragmentShader};