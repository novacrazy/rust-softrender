//! Minimalist framebuffer structure with an emphasis on performance

use std::ops::{Deref, DerefMut};

use ::pixel::Pixel;
use ::blend::Blend;
use ::texture::Texture;

/// Default depth value, equal to the farthest away anything can be.
///
/// Note that due to how floating point numbers work,
/// depth values become less precise the farther away the object is.
pub const DEFAULT_DEPTH_VALUE: f32 = ::std::f32::MAX;

pub enum StencilTest {
    Always,
    Never,
    LessThan,
    GreaterThan,
    LessThanEq,
    GreaterThanEq,
    Equal,
    NotEqual,
}

impl StencilTest {
    pub fn test(&self, present: u8, value: u8) -> bool {
        match *self {
            StencilTest::Always => true,
            StencilTest::Never => false,
            StencilTest::LessThan => value < present,
            StencilTest::LessThanEq => value <= present,
            StencilTest::GreaterThan => value > present,
            StencilTest::GreaterThanEq => value >= present,
            StencilTest::Equal => value == present,
            StencilTest::NotEqual => value != present,
        }
    }
}

pub enum StencilOp {
    Keep,
    Invert,
    Zero,
    Replace(u8),
    Increment(bool),
    Decrement(bool),
}

impl StencilOp {
    pub fn op(&self, value: u8) -> u8 {
        match *self {
            StencilOp::Keep => value,
            StencilOp::Invert => !value,
            StencilOp::Zero => 0,
            StencilOp::Replace(replacement) => replacement,
            StencilOp::Increment(true) => value.wrapping_add(1),
            StencilOp::Increment(false) => value.saturating_add(1),
            StencilOp::Decrement(true) => value.wrapping_sub(1),
            StencilOp::Decrement(false) => value.saturating_sub(1),
        }
    }
}

struct Components<P: Pixel> {
    attachments: P,
    depth: f32,
    stencil: u8,
}

pub struct FrameBuffer<P: Pixel> {
    width: u32,
    height: u32,
    attachments: Vec<P>,
    stencil_test: StencilTest,
    stencil_op: StencilOp,
}

impl<P: Pixel> FrameBuffer<P> {
    /// Create a new framebuffer with the default pixel.
    pub fn new(width: u32, height: u32) -> FrameBuffer<P> where P: Default {
        Self::new_with(width, height, Default::default())
    }

    /// Creates a new framebuffer with the given pixel.
    pub fn new_with(width: u32, height: u32, pixel: P) -> FrameBuffer<P> {
        let len = width as usize * height as usize;
        FrameBuffer {
            width,
            height,
            attachments: vec![pixel; len],
            stencil_test: StencilTest::Always,
            stencil_op: StencilOp::Keep,
        }
    }

    /// Get the width of the framebuffer in pixels
    #[inline(always)]
    pub fn width(&self) -> u32 { self.width }

    /// Get the height of the framebuffer in pixels
    #[inline(always)]
    pub fn height(&self) -> u32 { self.height }

    pub fn as_texture<'a, C: Pixel, F>(&'a self, component: F) -> FrameBufferTexture<'a, P, C, F> where F: Fn(&'a P) -> &'a C {
        FrameBufferTexture {
            framebuffer: self,
            get_component: component
        }
    }
}

pub struct FrameBufferTexture<'a, P: Pixel, C: Pixel, F> where F: Fn(&'a P) -> &'a C {
    framebuffer: &'a FrameBuffer<P>,
    get_component: F
}

impl<'a, P: Pixel, C: Pixel, F> Texture<C> for FrameBufferTexture<'a, P, C, F> where F: Fn(&'a P) -> &'a C {
    fn width(&self) -> u32 { self.framebuffer.width() }
    fn height(&self) -> u32 { self.framebuffer.height() }

    fn pixel(&self, x: u32, y: u32) -> &C {
        (self.get_component)(
            &self.framebuffer.attachments[(x + y * self.width()) as usize]
        )
    }
}