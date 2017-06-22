use super::{Framebuffer, Dimensions, Attachments};
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

impl<A> Framebuffer for RenderBuffer<A> where A: Attachments {
    type Attachments = A;

    #[inline]
    fn dimensions(&self) -> Dimensions { self.dimensions }

    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color) {
        for mut a in &mut self.buffer {
            *a = (color, <A::Depth as Depth>::far(), Default::default());
        }
    }

    #[inline]
    unsafe fn get_pixel_unchecked(&self, index: usize) -> <Self::Attachments as Attachments>::Color {
        self.buffer.get_unchecked(index).0
    }

    #[inline]
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: <Self::Attachments as Attachments>::Color) {
        self.buffer.get_unchecked_mut(index).0 = color;
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