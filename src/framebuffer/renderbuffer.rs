use ::geometry::{Dimensions, HasDimensions};
use ::pixel::{PixelBuffer, PixelRead, PixelWrite};

use super::{Framebuffer, Attachments};
use super::attachments::{Color, Depth, Stencil};

/// Renderbuffer framebuffer with interleaved attachments, allowing for more cache locality but
/// it cannot be re-used later as a texture without copying the attachments out.
#[derive(Clone)]
pub struct RenderBuffer<A: Attachments> {
    dimensions: Dimensions,
    stencil: <A::Stencil as Stencil>::Config,
    /// Interlaced framebuffer for more cache-friendly access
    pub ( crate ) buffer: Vec<(A::Color, A::Depth, <A::Stencil as Stencil>::Type)>,
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
        self.buffer.get_unchecked(index).0
    }
}

impl<A: Attachments> PixelWrite for RenderBuffer<A> {
    #[inline]
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Self::Color) {
        self.buffer.get_unchecked_mut(index).0 = color;
    }
}

impl<A: Attachments> Framebuffer for RenderBuffer<A> {
    type Attachments = A;

    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color) {
        for mut a in &mut self.buffer {
            *a = (color, <A::Depth as Depth>::far(), Default::default());
        }
    }
}

impl<A: Attachments> RenderBuffer<A> {
    pub fn new() -> RenderBuffer<A> {
        RenderBuffer {
            dimensions: Dimensions::new(0, 0),
            stencil: Default::default(),
            buffer: Vec::new()
        }
    }

    pub fn with_dimensions(dimensions: Dimensions) -> RenderBuffer<A> {
        RenderBuffer {
            dimensions,
            stencil: Default::default(),
            buffer: vec![(<A::Color as Color>::empty(),
                          <A::Depth as Depth>::far(),
                          Default::default());
                         dimensions.pixels()]
        }
    }
}