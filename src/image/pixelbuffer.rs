use super::image::{Pixel, GenericImage, Primitive, Rgb, Luma, Rgba, LumaA};

use ::behavior::ThreadSafeCopyable;
use ::geometry::{Coordinate, Dimensions, HasDimensions};
use ::pixel::{PixelBuffer, PixelRead, PixelWrite};
use ::color::Color;

impl<T> HasDimensions for T where T: GenericImage {
    fn dimensions(&self) -> Dimensions {
        let (width, height) = <T as GenericImage>::dimensions(self);
        Dimensions::new(width, height)
    }
}

impl<T> PixelBuffer for T where T: GenericImage, <T as GenericImage>::Pixel: Color {
    type Color = <T as GenericImage>::Pixel;
}

impl<T> PixelRead for T where T: GenericImage, <T as GenericImage>::Pixel: Color {
    #[inline]
    unsafe fn get_pixel_unchecked(&self, index: usize) -> Self::Color {
        let Coordinate { x, y } = Coordinate::from_index(index, <Self as HasDimensions>::dimensions(self));
        self.unsafe_get_pixel(x, y)
    }
}

impl<T> PixelWrite for T where T: GenericImage, <T as GenericImage>::Pixel: Color {
    #[inline]
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Self::Color) {
        let Coordinate { x, y } = Coordinate::from_index(index, <Self as HasDimensions>::dimensions(self));
        self.unsafe_put_pixel(x, y, color);
    }
}