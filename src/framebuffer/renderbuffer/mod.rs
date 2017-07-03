//! An efficient framebuffer implementation

use ::geometry::{Dimensions, HasDimensions};
use ::pixels::{PixelBuffer, PixelRead, PixelWrite};

use super::{FramebufferBase, UnsafeFramebuffer, Framebuffer, Attachments};
use super::attachments::{Color, Depth, Stencil};
use super::types::{ColorAttachment, DepthAttachment, StencilAttachment};

pub mod iterator;

pub use self::iterator::{RenderBufferIter, RenderBufferIterMut};

/// Interlaced framebuffer for more cache-friendly access
#[derive(Debug, Clone, Copy)]
pub ( in ::framebuffer::renderbuffer) struct RenderBufferAttachments<A: Attachments> {
    color: A::Color,
    depth: A::Depth,
    stencil: A::Stencil,
}

impl<A: Attachments> Default for RenderBufferAttachments<A> {
    fn default() -> RenderBufferAttachments<A> {
        RenderBufferAttachments {
            color: Color::empty(),
            depth: Depth::far(),
            stencil: Default::default(),
        }
    }
}

/// An efficient framebuffer implementation with interleaved attachments, allowing for more cache locality but
/// it cannot be re-used later as a texture without copying the attachments out.
pub struct RenderBuffer<A: Attachments> {
    dimensions: Dimensions,
    buffer: Vec<RenderBufferAttachments<A>>,
}

impl<A: Attachments> Clone for RenderBuffer<A> {
    fn clone(&self) -> RenderBuffer<A> {
        RenderBuffer {
            buffer: self.buffer.clone(),
            ..*self
        }
    }
}

impl<A: Attachments> RenderBuffer<A> {
    /// Create a new empty `RenderBuffer` with no allocated pixels.
    pub fn new() -> RenderBuffer<A> {
        RenderBuffer {
            dimensions: Dimensions::new(0, 0),
            buffer: Vec::new()
        }
    }

    /// Create a new empty `Renderbuffer` with the given number of pixels allocated.
    pub fn with_dimensions(dimensions: Dimensions) -> RenderBuffer<A> {
        RenderBuffer {
            dimensions,
            buffer: vec![RenderBufferAttachments::default(); dimensions.area()]
        }
    }

    /// Return an efficient iterator for `RenderBuffer` pixels
    pub fn iter<'a>(&'a self) -> RenderBufferIter<'a, A> {
        RenderBufferIter { iter: self.buffer.iter() }
    }

    /// Return an efficient iterator for mutating `RenderBuffer` pixels
    pub fn iter_mut<'a>(&'a mut self) -> RenderBufferIterMut<'a, A> {
        RenderBufferIterMut { iter: self.buffer.iter_mut() }
    }
}

impl<A: Attachments> HasDimensions for RenderBuffer<A> {
    #[inline]
    fn dimensions(&self) -> Dimensions { self.dimensions }
}

impl<A: Attachments> PixelBuffer for RenderBuffer<A> {
    type Color = <A as Attachments>::Color;
}

impl<A: Attachments> PixelRead for RenderBuffer<A> {
    #[inline]
    unsafe fn get_pixel_unchecked(&self, index: usize) -> Self::Color {
        self.buffer.get_unchecked(index).color
    }
}

impl<A: Attachments> PixelWrite for RenderBuffer<A> {
    #[inline]
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Self::Color) {
        self.buffer.get_unchecked_mut(index).color = color;
    }
}

impl<A: Attachments> FramebufferBase for RenderBuffer<A> {
    type Attachments = A;
}

impl<A: Attachments> UnsafeFramebuffer for RenderBuffer<A> {
    #[inline]
    unsafe fn get_depth_unchecked(&self, index: usize) -> DepthAttachment<Self> {
        self.buffer.get_unchecked(index).depth
    }

    #[inline]
    unsafe fn set_depth_unchecked(&mut self, index: usize, depth: DepthAttachment<Self>) {
        self.buffer.get_unchecked_mut(index).depth = depth;
    }

    #[inline]
    unsafe fn get_stencil_unchecked(&self, index: usize) -> StencilAttachment<Self> {
        self.buffer.get_unchecked(index).stencil
    }

    #[inline]
    unsafe fn set_stencil_unchecked(&mut self, index: usize, stencil: StencilAttachment<Self>) {
        self.buffer.get_unchecked_mut(index).stencil = stencil;
    }
}

impl<A: Attachments> Framebuffer for RenderBuffer<A> {
    fn clear(&mut self, color: ColorAttachment<Self>) {
        for mut a in &mut self.buffer {
            *a = RenderBufferAttachments {
                color,
                ..RenderBufferAttachments::default()
            };
        }
    }
}