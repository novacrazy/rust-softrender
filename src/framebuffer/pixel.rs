use super::{Framebuffer, Attachments};

/// Immutable reference to a pixel.
///
/// Provides a read-only accessor for the pixel at the coordinates given at creation.
pub struct PixelRef<'a, F>(usize, &'a F) where F: Framebuffer;

/// Mutable reference to a pixel
///
/// Provides a writable accessor for the pixel at the coordinates given at creation.
pub struct PixelMut<'a, F>(usize, &'a mut F) where F: Framebuffer;

impl<'a, F> PixelRef<'a, F> where F: Framebuffer {
    #[inline(always)]
    pub ( in ::framebuffer) fn new(index: usize, framebuffer: &'a F) -> PixelRef<'a, F> {
        PixelRef(index, framebuffer)
    }

    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <<F as Framebuffer>::Attachments as Attachments>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }
}

impl<'a, F> PixelMut<'a, F> where F: Framebuffer {
    #[inline(always)]
    pub ( in ::framebuffer) fn new(index: usize, framebuffer: &'a mut F) -> PixelMut<'a, F> {
        PixelMut(index, framebuffer)
    }

    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <<F as Framebuffer>::Attachments as Attachments>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }

    /// Set the pixel
    #[inline]
    pub fn set(&mut self, color: <<F as Framebuffer>::Attachments as Attachments>::Color) {
        unsafe { self.1.set_pixel_unchecked(self.0, color) }
    }

    #[inline]
    pub fn into_ref(self) -> PixelRef<'a, F> {
        PixelRef(self.0, self.1)
    }
}

impl<'a, F> From<PixelMut<'a, F>> for PixelRef<'a, F> where F: Framebuffer {
    #[inline]
    fn from(pixel: PixelMut<'a, F>) -> PixelRef<'a, F> { pixel.into_ref() }
}