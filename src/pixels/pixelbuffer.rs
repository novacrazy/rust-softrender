//! Pixel accessor structures
use ::error::{RenderResult, RenderError};
use ::color::Color;
use ::geometry::{Dimensions, Coordinate, HasDimensions};

/*
/// Partial `PixelBuffer` taken from a full `PixelBuffer`,
/// allowing operations on subspaces.
pub struct SubPixelBuffer<'a, P: 'a> {
    parent: &'a P,
    start: Coordinate,
    end: Coordinate,
}

impl<'a, P: 'a> SubPixelBuffer<'a, P> {
    /// The start, or bottom left corner, of the partial `PixelBuffer` in relation to the parent `PixelBuffer`
    #[inline]
    pub fn start(&self) -> Coordinate { self.start }

    /// The end, or top right corner, of the partial `PixelBuffer` in relation to the parent `PixelBuffer`
    #[inline]
    pub fn end(&self) -> Coordinate { self.end }

    /// Reference to the parent `PixelBuffer`
    #[inline]
    pub fn parent(&self) -> &'a P { self.parent }
}

impl<'a, P: 'a> HasDimensions for SubPixelBuffer<'a, P> {
    fn dimensions(&self) -> Dimensions {
        self.end - self.start
    }
}

impl<'a, P: 'a> PixelBuffer for SubPixelBuffer<'a, P> where P: PixelBuffer {
    type Color = <P as PixelBuffer>::Color;
}

impl<'a, P: 'a> PixelRead for SubPixelBuffer<'a, P> where P: PixelRead {
    unsafe fn get_pixel_unchecked(&self, index: usize) -> Self::Color {
        self.parent.get_pixel_unchecked(index)
    }
}
*/

/// Generic buffer type trait, which defines the `Color` type for any pixel in the buffer
pub trait PixelBuffer: Sized + HasDimensions {
    type Color: Color;

    //fn sub_pixelbuffer(&self, start: Coordinate, end: Coordinate) -> RenderResult<SubPixelBuffer<Self>> {
    //    if start < end {
    //        Ok(SubPixelBuffer { parent: self, start, end })
    //    } else {
    //        throw!(RenderError::InvalidPixelCoordinate);
    //    }
    //}
}

/// Defines unsafe methods for reading raw pixel values.
///
/// These are meant to have little to no overhead,
/// where the safe abstractions are `PixelRef`/`PixelMut`.
pub trait PixelRead: PixelBuffer {
    /// Get pixel value without checking bounds.
    unsafe fn get_pixel_unchecked(&self, index: usize) -> Self::Color;

    /// Get a "reference" to the pixel at the given coordinate.
    ///
    /// Throws `RenderError::InvalidPixelCoordinate` on invalid pixel coordinates.
    fn pixel_ref<'a>(&'a self, coord: Coordinate) -> RenderResult<PixelRef<'a, Self>> {
        let dim = self.dimensions();

        if dim.in_bounds(coord) {
            Ok(PixelRef::new(coord.into_index(dim), self))
        } else {
            throw!(RenderError::InvalidPixelCoordinate);
        }
    }
}

/// Defines unsafe methods for writing to raw pixel values.
///
/// These are meant to have little to no overhead,
/// where the safe abstractions are `PixelRef`/`PixelMut`.
pub trait PixelWrite: PixelRead {
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Self::Color);

    /// Get a mutable "reference" to the pixel at the given coordinate.
    ///
    /// Throws `RenderError::InvalidPixelCoordinate` on invalid pixel coordinates.
    fn pixel_mut<'a>(&'a mut self, coord: Coordinate) -> RenderResult<PixelMut<'a, Self>> {
        let dim = self.dimensions();

        if dim.in_bounds(coord) {
            Ok(PixelMut::new(coord.into_index(dim), self))
        } else {
            throw!(RenderError::InvalidPixelCoordinate);
        }
    }
}

/// Immutable reference to a pixel.
///
/// Provides a read-only accessor for the pixel at the coordinates given at creation.
pub struct PixelRef<'a, T: 'a>(usize, &'a T) where T: PixelRead;

/// Mutable reference to a pixel
///
/// Provides a writable accessor for the pixel at the coordinates given at creation.
pub struct PixelMut<'a, T: 'a>(usize, &'a mut T) where T: PixelWrite;

impl<'a, T: 'a> PixelRef<'a, T> where T: PixelRead {
    #[inline(always)]
    pub ( crate ) fn new(index: usize, framebuffer: &'a T) -> PixelRef<'a, T> {
        PixelRef(index, framebuffer)
    }

    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <T as PixelBuffer>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }
}

impl<'a, T: 'a> PixelMut<'a, T> where T: PixelWrite {
    #[inline(always)]
    pub ( crate ) fn new(index: usize, framebuffer: &'a mut T) -> PixelMut<'a, T> {
        PixelMut(index, framebuffer)
    }

    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <T as PixelBuffer>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }

    /// Set the pixel
    #[inline]
    pub fn set(&mut self, color: <T as PixelBuffer>::Color) {
        unsafe { self.1.set_pixel_unchecked(self.0, color) }
    }

    /// Downcast the current `PixelMut` into an immutable `PixelRef`.
    #[inline]
    pub fn into_ref(self) -> PixelRef<'a, T> {
        PixelRef(self.0, self.1)
    }
}

impl<'a, T: 'a> From<PixelMut<'a, T>> for PixelRef<'a, T> where T: PixelWrite {
    #[inline]
    fn from(pixel: PixelMut<'a, T>) -> PixelRef<'a, T> { pixel.into_ref() }
}