//! Black-hole Framebuffer

use ::geometry::{Dimensions, HasDimensions};
use ::pixels::{PixelBuffer, PixelRead, PixelWrite};

use super::{FramebufferBase, UnsafeFramebuffer, Framebuffer, attachments};

/// Black-hole Framebuffer that stores no pixels but still has dimensions.
///
/// Reading/writing to this framebuffer will do absolutely nothing.
#[derive(Debug, Clone, Copy)]
pub struct NullFramebuffer {
    dimensions: Dimensions,
}

impl HasDimensions for NullFramebuffer {
    #[inline]
    fn dimensions(&self) -> Dimensions { self.dimensions }
}

impl PixelBuffer for NullFramebuffer {
    type Color = ();
}

impl PixelRead for NullFramebuffer {
    #[inline(always)]
    unsafe fn get_pixel_unchecked(&self, _: usize) -> () { () }
}

impl PixelWrite for NullFramebuffer {
    #[inline(always)]
    unsafe fn set_pixel_unchecked(&mut self, _: usize, _: ()) {}
}

impl FramebufferBase for NullFramebuffer {
    type Attachments = attachments::predefined::EmptyAttachments;
}

impl UnsafeFramebuffer for NullFramebuffer {
    #[inline(always)]
    unsafe fn get_depth_unchecked(&self, _: usize) -> () { () }
    #[inline(always)]
    unsafe fn set_depth_unchecked(&mut self, _: usize, _: ()) {}

    #[inline(always)]
    unsafe fn get_stencil_unchecked(&self, _: usize) -> () { () }
    #[inline(always)]
    unsafe fn set_stencil_unchecked(&mut self, _: usize, _: ()) {}
}

impl Framebuffer for NullFramebuffer {
    #[inline(always)]
    fn clear(&mut self, _: ()) {}
}

impl NullFramebuffer {
    /// Create a new `NullFramebuffer` with no size
    #[inline]
    pub fn new() -> NullFramebuffer {
        NullFramebuffer::with_dimensions(Dimensions::new(0, 0))
    }

    /// Create a new `NullFramebuffer` with the given dimensions
    pub fn with_dimensions(dimensions: Dimensions) -> NullFramebuffer {
        NullFramebuffer { dimensions }
    }
}