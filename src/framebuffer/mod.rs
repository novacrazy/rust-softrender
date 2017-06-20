pub mod attachments;

pub use self::attachments::Attachments;

use self::attachments::Stencil;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    #[inline]
    pub fn new(width: u32, height: u32) -> Dimensions {
        Dimensions { width, height }
    }

    #[inline]
    pub fn pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[inline]
    pub fn valid(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }
}

pub trait Framebuffer: 'static {
    type Attachments: Attachments;
    fn dimensions(&self) -> Dimensions;
}

pub struct RenderBuffer<A: Attachments> {
    dimensions: Dimensions,
    stencil: <A::Stencil as Stencil>::Config,
    /// Interlaced framebuffer for more cache-friendly access
    pub ( crate ) buffer: Vec<(A::Color,
                               A::Depth,
                               <A::Stencil as Stencil>::Type)>,
}

impl<A> Framebuffer for RenderBuffer<A> where A: Attachments {
    type Attachments = A;

    fn dimensions(&self) -> Dimensions { self.dimensions }
}
