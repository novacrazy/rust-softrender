use ::error::{RenderError, RenderResult};

use ::color::Color;
use ::pixel::{PixelBuffer, PixelRead, PixelWrite};
use ::geometry::{Dimensions, Coordinate};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    Nearest,
    Bilinear,
}

impl Default for Filter {
    fn default() -> Filter {
        Filter::Nearest
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Edge<C: Color> {
    /// Equivalent to `GL_CLAMP_TO_EDGE`
    Clamp,
    /// Equivalent to `GL_REPEAT`
    Wrap,
    /// Equivalent to `GL_CLAMP_TO_BORDER`
    Border(C)
}

impl<C: Color> Default for Edge<C> {
    fn default() -> Edge<C> { Edge::Clamp }
}

pub trait Texture: PixelBuffer {}

pub trait TextureRead: Texture + PixelRead {
    /// Samples a pixel from a floating-point coordinate, applying the selected `Filter` and `Edge` behavior.
    fn sample(&self, _x: f32, _y: f32, _filter: Filter, _edge: Edge<Self::Color>) -> RenderResult<Self::Color> {
        unimplemented!()
    }
}

pub trait TextureWrite: Texture + PixelWrite {
    /// "Unsamples", or writes, to a floating-point coordinate, applying the selected `Filter` and `Edge` behavior.
    ///
    /// This allows writing to multiple pixels based on fractional coordinates.
    fn unsample(&mut self, _x: f32, _y: f32, _filter: Filter, _edge: Edge<Self::Color>) -> RenderResult<()> {
        unimplemented!()
    }
}

//pub struct SliceTexture<'a, C: Color + 'a> {
//    slice: &'a C,
//    dimensions: Dimensions,
//}