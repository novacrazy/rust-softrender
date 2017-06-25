//! Pixel accessor structures
use std::slice;

use ::error::{RenderResult, RenderError};
use ::color::Color;
use ::geometry::{Dimensions, Coordinate, HasDimensions};

pub mod iterator;

pub use self::iterator::PixelBufferIter;

/// Generic buffer type trait, which defines the `Color` type for any pixel in the buffer
pub trait PixelBuffer: Sized + HasDimensions {
    type Color: Color;
}


/// Defines methods for reading raw pixel values.
pub trait PixelRead: PixelBuffer {
    /// Unsafely access a pixel at the given index without checking bounds.
    ///
    /// This is meant for internal use, do not attempt to use it directly. Please use
    /// `pixel_ref` or `pixel_iter` to access pixel values safely.
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

    /// Efficient iterator for accessing all pixels in the buffer.
    ///
    /// However, certain framebuffers or pixelbuffers may provide their
    /// own iterators for even more efficient access.
    fn pixel_iter<'a>(&'a self) -> PixelBufferIter<'a, Self> {
        PixelBufferIter {
            buffer: self,
            position: 0,
            max_len: self.dimensions().area()
        }
    }
}

/// Defines methods for writing to raw pixel values.
pub trait PixelWrite: PixelRead {
    /// Unsafely access a pixel at the given index without checking bounds.
    ///
    /// This is meant for internal use, do not attempt to use it directly. Please use
    /// `pixel_mut` to access pixel values safely.
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
pub struct PixelRef<'a, P: 'a>(pub ( in ::pixels) usize,
                               pub ( in ::pixels) &'a P) where P: PixelRead;

/// Mutable reference to a pixel
///
/// Provides a writable accessor for the pixel at the coordinates given at creation.
pub struct PixelMut<'a, P: 'a>(pub ( in ::pixels) usize,
                               pub ( in ::pixels) &'a mut P) where P: PixelWrite;

impl<'a, P: 'a> PixelRef<'a, P> where P: PixelRead {
    #[inline(always)]
    pub ( in ::pixels ) fn new(index: usize, framebuffer: &'a P) -> PixelRef<'a, P> {
        PixelRef(index, framebuffer)
    }

    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <P as PixelBuffer>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }
}

impl<'a, P: 'a> PixelMut<'a, P> where P: PixelWrite {
    #[inline(always)]
    pub ( in ::pixels ) fn new(index: usize, framebuffer: &'a mut P) -> PixelMut<'a, P> {
        PixelMut(index, framebuffer)
    }

    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <P as PixelBuffer>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }

    /// Set the pixel
    #[inline]
    pub fn set(&mut self, color: <P as PixelBuffer>::Color) {
        unsafe { self.1.set_pixel_unchecked(self.0, color) }
    }

    /// Downcast the current `PixelMut` into an immutable `PixelRef`.
    #[inline]
    pub fn into_ref(self) -> PixelRef<'a, P> {
        PixelRef(self.0, self.1)
    }
}

impl<'a, P: 'a> From<PixelMut<'a, P>> for PixelRef<'a, P> where P: PixelWrite {
    #[inline]
    fn from(pixel: PixelMut<'a, P>) -> PixelRef<'a, P> { pixel.into_ref() }
}