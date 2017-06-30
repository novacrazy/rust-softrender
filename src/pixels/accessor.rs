use super::{PixelBuffer, PixelRead, PixelWrite};

/// Immutable reference to a pixel.
///
/// Provides a read-only accessor for the pixel at the coordinates given at creation.
pub struct PixelRef<'a, P: 'a>(pub ( in ::pixels) usize,
                               pub ( in ::pixels) &'a P) where P: PixelRead;

impl<'a, P: 'a> Clone for PixelRef<'a, P> where P: PixelRead {
    fn clone(&self) -> PixelRef<'a, P> {
        PixelRef { ..*self }
    }
}

impl<'a, P: 'a> Copy for PixelRef<'a, P> where P: PixelRead {}

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