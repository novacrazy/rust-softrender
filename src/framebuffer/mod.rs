use ::error::RenderResult;

pub mod attachments;
pub mod dimension;
pub mod coordinate;
pub mod pixel;
pub mod nullbuffer;
pub mod renderbuffer;
pub mod texturebuffer;

pub use self::attachments::Attachments;
pub use self::dimension::Dimensions;
pub use self::coordinate::Coordinate;
pub use self::renderbuffer::RenderBuffer;

use self::attachments::{Depth, Stencil};
use self::pixel::{PixelRef, PixelMut};

pub trait Framebuffer: Sized + Clone + 'static {
    /// Associated type for the framebuffer attachments
    type Attachments: Attachments;

    /// Returns the dimensions of the framebuffer
    fn dimensions(&self) -> Dimensions;

    /// Clears the framebuffer with the given color, and sets any depth or stencil buffers back to their default values.
    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color);

    /// Get pixel value without checking bounds.
    ///
    /// WARNING: This might segfault on an invalid index. Please use `pixel_ref`/`pixel_mut` for checked pixel access
    unsafe fn get_pixel_unchecked(&self, index: usize) -> <Self::Attachments as Attachments>::Color;

    /// Set pixel value without checking bounds.
    ///
    /// WARNING: This might segfault on an invalid index. Please use `pixel_ref`/`pixel_mut` for checked pixel access
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: <Self::Attachments as Attachments>::Color);

    /// Get a "reference" to the pixel at the given coordinate. Throws `RenderError::InvalidPixelCoordinate` on invalid pixel coordinates.
    #[inline]
    fn pixel_ref<'a>(&'a self, coord: Coordinate) -> RenderResult<PixelRef<'a, Self>> {
        self.dimensions().check_valid(coord).map(|_| {
            PixelRef::new(coord.into_index(), self)
        })
    }

    /// Get a mutable "reference" to the pixel at the given coordinate. Throws `RenderError::InvalidPixelCoordinate` on invalid pixel coordinates.
    #[inline]
    fn pixel_mut<'a>(&'a mut self, coord: Coordinate) -> RenderResult<PixelMut<'a, Self>> {
        self.dimensions().check_valid(coord).map(move |_| {
            PixelMut::new(coord.into_index(), self)
        })
    }
}