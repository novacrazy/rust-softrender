//! Pixel accessor structures
use ::error::RenderResult;
use ::color::Color;
use ::geometry::{Coordinate, HasDimensions};

/// Generic buffer type trait, which defines the `Color` type for any pixel in the buffer
pub trait PixelBuffer: Sized + HasDimensions {
    type Color: Color;
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
        self.dimensions().check_valid(coord).map(|_| {
            PixelRef::new(coord.into_index(self.dimensions()), self)
        })
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
        self.dimensions().check_valid(coord).map(move |_| {
            PixelMut::new(coord.into_index(self.dimensions()), self)
        })
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