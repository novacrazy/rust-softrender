//! Rendering pipeline implementation

pub mod blend;
pub mod geometry;
pub mod line;
pub mod framebuffer;
pub mod uniform;
pub mod pipeline;

pub use self::blend::GenericBlend;
pub use self::geometry::{FaceWinding, ClipVertex, ScreenVertex};
pub use self::framebuffer::FrameBuffer;
pub use self::uniform::{Barycentric, barycentric_interpolate};
pub use self::pipeline::{Pipeline, VertexShader, FragmentShader, Fragment, LineStyle};