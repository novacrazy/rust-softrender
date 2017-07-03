pub mod attachments;
pub mod types;
pub mod accessor;
pub mod nullbuffer;
pub mod renderbuffer;
pub mod texturebuffer;

pub use self::attachments::Attachments;
pub use self::renderbuffer::RenderBuffer;

use ::error::{RenderResult, RenderError};

use ::geometry::{Coordinate, HasDimensions};
use ::pixels::PixelWrite;

use self::types::{ColorAttachment, DepthAttachment, StencilAttachment};
use self::accessor::{FramebufferAccessor, FramebufferAccessorMut};

/// Framebuffer base trait defining any attachments and prerequisite traits
pub trait FramebufferBase: Sized + Clone + HasDimensions + PixelWrite + 'static {
    /// Associated type for the framebuffer attachments
    type Attachments: Attachments;
}

/// Unsafe Framebuffer trait defining all unsafe methods for internal use
pub trait UnsafeFramebuffer: FramebufferBase {
    unsafe fn get_depth_unchecked(&self, index: usize) -> DepthAttachment<Self>;
    unsafe fn set_depth_unchecked(&mut self, index: usize, depth: DepthAttachment<Self>);

    unsafe fn get_stencil_unchecked(&self, index: usize) -> StencilAttachment<Self>;
    unsafe fn set_stencil_unchecked(&mut self, index: usize, stencil: StencilAttachment<Self>);
}

/// Standard Framebuffer trait defining user-facing methods
pub trait Framebuffer: UnsafeFramebuffer {
    /// Clears the framebuffer with the given color, and sets any depth or stencil buffers back to their default values.
    fn clear(&mut self, color: ColorAttachment<Self>);

    fn attachments(&self, coord: Coordinate) -> RenderResult<FramebufferAccessor<Self>> {
        let dim = self.dimensions();

        if dim.in_bounds(coord) {
            Ok(FramebufferAccessor::new(coord.into_index(dim), self))
        } else {
            throw!(RenderError::InvalidPixelCoordinate);
        }
    }

    fn attachments_mut(&mut self, coord: Coordinate) -> RenderResult<FramebufferAccessorMut<Self>> {
        let dim = self.dimensions();

        if dim.in_bounds(coord) {
            Ok(FramebufferAccessorMut::new(coord.into_index(dim), self))
        } else {
            throw!(RenderError::InvalidPixelCoordinate);
        }
    }
}