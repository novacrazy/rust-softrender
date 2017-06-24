use super::image::{Pixel, GenericImage, Primitive, Rgb, Luma, Rgba, LumaA};

use ::behavior::ThreadSafeCopyable;
use ::geometry::{Coordinate, Dimensions, HasDimensions};
use ::pixel::{PixelBuffer, PixelRead, PixelWrite};
use ::color::Color;

impl<T: Primitive> Color for Rgb<T> where T: ThreadSafeCopyable + Default {
    type Alpha = ();

    #[inline]
    fn empty() -> Rgb<T> {
        Rgb { data: [T::zero(); 3] }
    }

    #[inline]
    fn with_alpha(self, _: ()) -> Self { self }

    #[inline]
    fn mul_alpha(self, _: ()) -> Self { self }

    #[inline]
    fn get_alpha(&self) -> () { () }
}

impl<T: Primitive> Color for Luma<T> where T: ThreadSafeCopyable + Default {
    type Alpha = ();

    #[inline]
    fn empty() -> Luma<T> {
        Luma { data: [T::zero(); 1] }
    }

    #[inline]
    fn with_alpha(self, _: ()) -> Self { self }

    #[inline]
    fn mul_alpha(self, _: ()) -> Self { self }

    #[inline]
    fn get_alpha(&self) -> () { () }
}

impl<T: Primitive> Color for Rgba<T> where T: ThreadSafeCopyable + Default {
    type Alpha = T;

    #[inline]
    fn empty() -> Rgba<T> {
        Rgba { data: [T::zero(); 4] }
    }

    fn with_alpha(self, alpha: T) -> Self {
        Rgba {
            data: [
                self.data[0],
                self.data[1],
                self.data[2],
                alpha
            ]
        }
    }

    fn mul_alpha(self, alpha: T) -> Self {
        Rgba {
            data: [
                self.data[0],
                self.data[1],
                self.data[2],
                self.data[3] * alpha
            ]
        }
    }

    #[inline]
    fn get_alpha(&self) -> T {
        self.data[3]
    }
}

impl<T: Primitive> Color for LumaA<T> where T: ThreadSafeCopyable + Default {
    type Alpha = T;

    #[inline]
    fn empty() -> LumaA<T> {
        LumaA { data: [T::zero(); 2] }
    }

    fn with_alpha(self, alpha: T) -> Self {
        LumaA { data: [self.data[0], alpha] }
    }

    fn mul_alpha(self, alpha: T) -> Self {
        LumaA { data: [self.data[0], self.data[1] * alpha] }
    }

    #[inline]
    fn get_alpha(&self) -> T {
        self.data[1]
    }
}