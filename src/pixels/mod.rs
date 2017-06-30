//! Pixel accessor structures
use ::error::{RenderResult, RenderError};
use ::color::Color;
use ::geometry::{Coordinate, HasDimensions};

pub mod accessor;
pub mod iterator;
pub mod partial;

pub use self::iterator::PixelBufferIter;

pub use self::partial::{PartialPixelBuffer, PartialPixelBufferRef, PartialPixelBufferMut};

use self::accessor::{PixelRef, PixelMut};

/// Generic buffer type trait, which defines the `Color` type for any pixel in the buffer
pub trait PixelBuffer: Sized + HasDimensions {
    type Color: Color;

    /// Returns a partial 2D "slice" of a pixelbuffer with most of the same properties.
    ///
    /// Partial pixelbuffers are slower than direct usage of their parent pixelbuffer as every pixel access requires
    /// computing the offset to the parent pixelbuffer.
    fn partial_ref(&self, start: Coordinate, end: Coordinate) -> RenderResult<PartialPixelBufferRef<Self>> {
        if start < end {
            Ok(PartialPixelBufferRef { parent: self, start, end })
        } else {
            throw!(RenderError::InvalidPixelCoordinate);
        }
    }

    /// Returns a mutable partial 2D "slice" of a pixelbuffer with most of the same properties.
    ///
    /// Partial pixelbuffers are slower than direct usage of their parent pixelbuffer as every pixel access requires
    /// computing the offset to the parent pixelbuffer.
    fn partial_mut(&mut self, start: Coordinate, end: Coordinate) -> RenderResult<PartialPixelBufferMut<Self>> {
        if start < end {
            Ok(PartialPixelBufferMut { parent: self, start, end })
        } else {
            throw!(RenderError::InvalidPixelCoordinate);
        }
    }
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
    fn pixel_ref(&self, coord: Coordinate) -> RenderResult<PixelRef<Self>> {
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
    fn pixel_iter(&self) -> PixelBufferIter<Self> {
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
    fn pixel_mut(&mut self, coord: Coordinate) -> RenderResult<PixelMut<Self>> {
        let dim = self.dimensions();

        if dim.in_bounds(coord) {
            Ok(PixelMut::new(coord.into_index(dim), self))
        } else {
            throw!(RenderError::InvalidPixelCoordinate);
        }
    }
}