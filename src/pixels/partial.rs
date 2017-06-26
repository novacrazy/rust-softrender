//! Partial PixelBuffers

use ::error::{RenderResult, RenderError};
use ::geometry::{Dimensions, HasDimensions, Coordinate};

use super::{PixelBuffer, PixelRead, PixelWrite, PixelRef, PixelMut};

/// Common properties and methods for `PartialPixelBuffer` types
pub trait PartialPixelBuffer: PixelBuffer {
    /// The parent pixelbuffer type
    type PixelBuffer;

    /// Relative to the parent pixelbuffer, the starting coordinate of the partial pixelbuffer
    fn start(&self) -> Coordinate;
    /// Relative to the parent pixelbuffer, the ending coordinate of the partial pixelbuffer
    fn end(&self) -> Coordinate;
    /// Reference to the parent pixelbuffer
    fn parent(&self) -> &Self::PixelBuffer;
}

/// Partial `PixelBuffer`, a sub-image of the parent `PixelBuffer`
pub struct PartialPixelBufferRef<'a, P: 'a> {
    pub ( in ::pixels) parent: &'a P,
    pub ( in ::pixels) start: Coordinate,
    pub ( in ::pixels) end: Coordinate,
}

/// Mutable partial `PixelBuffer`, a sub-image of the parent `PixelBuffer`
pub struct PartialPixelBufferMut<'a, P: 'a> {
    pub ( in ::pixels) parent: &'a mut P,
    pub ( in ::pixels) start: Coordinate,
    pub ( in ::pixels) end: Coordinate,
}

impl<'a, P: 'a> PartialPixelBuffer for PartialPixelBufferRef<'a, P> where P: PixelBuffer {
    type PixelBuffer = P;

    /// The start, or bottom left corner, of the partial `PixelBuffer` in relation to the parent `PixelBuffer`
    #[inline]
    fn start(&self) -> Coordinate { self.start }

    /// The end, or top right corner, of the partial `PixelBuffer` in relation to the parent `PixelBuffer`
    #[inline]
    fn end(&self) -> Coordinate { self.end }

    /// Reference to the parent `PixelBuffer`
    #[inline]
    fn parent(&self) -> &Self::PixelBuffer { self.parent }
}

impl<'a, P: 'a> PartialPixelBuffer for PartialPixelBufferMut<'a, P> where P: PixelBuffer {
    type PixelBuffer = P;

    /// The start, or bottom left corner, of the partial `PixelBuffer` in relation to the parent `PixelBuffer`
    #[inline]
    fn start(&self) -> Coordinate { self.start }

    /// The end, or top right corner, of the partial `PixelBuffer` in relation to the parent `PixelBuffer`
    #[inline]
    fn end(&self) -> Coordinate { self.end }

    /// Reference to the parent `PixelBuffer`
    #[inline]
    fn parent(&self) -> &Self::PixelBuffer { self.parent }
}

impl<'a, P: 'a> HasDimensions for PartialPixelBufferRef<'a, P> {
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.end.x - self.start.x,
            height: self.end.y - self.start.y
        }
    }
}

impl<'a, P: 'a> HasDimensions for PartialPixelBufferMut<'a, P> {
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.end.x - self.start.x,
            height: self.end.y - self.start.y
        }
    }
}

impl<'a, P: 'a> PixelBuffer for PartialPixelBufferRef<'a, P> where P: PixelBuffer {
    type Color = <P as PixelBuffer>::Color;
}

impl<'a, P: 'a> PixelRead for PartialPixelBufferRef<'a, P> where P: PixelRead {
    #[inline]
    unsafe fn get_pixel_unchecked(&self, index: usize) -> Self::Color {
        self.parent.get_pixel_unchecked(index)
    }

    fn pixel_ref<'b>(&'b self, coord: Coordinate) -> RenderResult<PixelRef<'b, Self>> {
        let PixelRef(index, _) = self.parent.pixel_ref(coord + self.start)?;
        Ok(PixelRef(index, self))
    }
}

impl<'a, P: 'a> PixelBuffer for PartialPixelBufferMut<'a, P> where P: PixelBuffer {
    type Color = <P as PixelBuffer>::Color;
}

impl<'a, P: 'a> PixelRead for PartialPixelBufferMut<'a, P> where P: PixelRead {
    #[inline]
    unsafe fn get_pixel_unchecked(&self, index: usize) -> Self::Color {
        self.parent.get_pixel_unchecked(index)
    }

    fn pixel_ref<'b>(&'b self, coord: Coordinate) -> RenderResult<PixelRef<'b, Self>> {
        let PixelRef(index, _) = self.parent.pixel_ref(coord + self.start)?;
        Ok(PixelRef(index, self))
    }
}

impl<'a, P: 'a> PixelWrite for PartialPixelBufferMut<'a, P> where P: PixelWrite {
    #[inline]
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Self::Color) {
        self.parent.set_pixel_unchecked(index, color);
    }

    fn pixel_mut<'b>(&'b mut self, coord: Coordinate) -> RenderResult<PixelMut<'b, Self>> {
        let PixelMut(index, _) = self.parent.pixel_mut(coord + self.start)?;
        Ok(PixelMut(index, self))
    }
}