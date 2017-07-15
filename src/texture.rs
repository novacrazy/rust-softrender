//! Texture handling
use nalgebra::Vector2;

use ::error::RenderResult;

use ::numeric::FloatScalar;
use ::color::Color;
use ::pixels::{PixelBuffer, PixelRead, PixelWrite};
use ::geometry::Coordinate;

pub type TextureColor<T> = <T as PixelBuffer>::Color;

/// A more traditional texture sampling method reminiscent of OpenGL.
pub fn texture<T: TextureRead, N: FloatScalar>(t: &T, coord: Vector2<N>,
                                               filter: Filter,
                                               edge: Edge<TextureColor<T>>) -> RenderResult<TextureColor<T>> {
    t.sample(coord, filter, edge)
}

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
    fn sample<N: FloatScalar>(&self, _coord: Vector2<N>, _filter: Filter, _edge: Edge<TextureColor<Self>>) -> RenderResult<TextureColor<Self>> {
        unimplemented!()
    }
}

pub trait TextureWrite: Texture + PixelWrite {
    /// "Unsamples", or writes, to a floating-point coordinate, applying the selected `Filter` and `Edge` behavior.
    ///
    /// This allows writing to multiple pixels based on fractional coordinates.
    fn unsample<N: FloatScalar>(&mut self, _coord: Vector2<N>, _filter: Filter, _edge: Edge<TextureColor<Self>>) -> RenderResult<()> {
        unimplemented!()
    }
}

impl<T> Texture for T where T: PixelBuffer {}

impl<T> TextureRead for T where T: Texture + PixelRead {}

impl<T> TextureWrite for T where T: Texture + PixelWrite {}

//pub struct SliceTexture<'a, C: Color + 'a> {
//    slice: &'a C,
//    dimensions: Dimensions,
//}