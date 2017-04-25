//! Rendering pipeline implementation

pub mod framebuffer;
pub mod projection;
pub mod rasterize;
pub mod uniform;
pub mod shading;

pub use self::framebuffer::FrameBuffer;
pub use self::shading::{ClipVertex, ScreenVertex};
pub use self::shading::Pipeline as ShadingPipeline;