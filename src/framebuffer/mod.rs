pub mod attachments;
pub mod nullbuffer;
pub mod renderbuffer;
pub mod texturebuffer;

pub use self::attachments::Attachments;
pub use self::renderbuffer::RenderBuffer;

use ::geometry::HasDimensions;
use ::pixels::PixelWrite;

pub trait Framebuffer: Sized + Clone + HasDimensions + PixelWrite + 'static {
    /// Associated type for the framebuffer attachments
    type Attachments: Attachments;

    /// Clears the framebuffer with the given color, and sets any depth or stencil buffers back to their default values.
    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color);
}