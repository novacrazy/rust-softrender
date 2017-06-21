//! Defines standard one to four channel colors of both `f32` and `u8` types.

use num_traits::{Num, Zero};

use nalgebra::{Vector1, Vector2, Vector3, Vector4, Scalar};
use nalgebra::coordinates::XYZW;

use super::Color;

pub mod formats {
    use nalgebra::{Vector1, Vector2, Vector3, Vector4};

    /// RGBA 32-bit Floating Point Color
    pub type RGBAf32Color = Vector4<f32>;
    /// RGB 32-bit Floating Point Color
    pub type RGBf32Color = Vector3<f32>;
    /// Red-Green 32-bit Floating Point Color
    pub type RGf32Color = Vector2<f32>;
    /// Red 32-bit Floating Point Color
    pub type Rf32Color = Vector1<f32>;

    /// RGBA 8-bit Unsigned Integer Color
    pub type RGBAu8Color = Vector4<u8>;
    /// RGB 8-bit Unsigned Integer Color
    pub type RGBu8Color = Vector3<u8>;
    /// Red-Green 8-bit Unsigned Integer Color
    pub type RGu8Color = Vector2<u8>;
    /// Red 8-bit Unsigned Integer Color
    pub type Ru8Color = Vector1<u8>;

    #[cfg(test)]
    mod test {
        use ::framebuffer::attachments::Color;
        use ::framebuffer::attachments::color::__assert_color;

        use super::*;

        #[test]
        fn test_f32_color_assert() {
            __assert_color::<RGBAf32Color>();
            __assert_color::<RGBf32Color>();
            __assert_color::<RGf32Color>();
            __assert_color::<Rf32Color>();
        }

        #[test]
        fn test_u8_color_assert() {
            __assert_color::<RGBAu8Color>();
            __assert_color::<RGBu8Color>();
            __assert_color::<RGu8Color>();
            __assert_color::<Ru8Color>();
        }
    }
}

impl<T> Color for Vector4<T> where T: Scalar + Num + Send + Sync + Default {
    type Alpha = T;

    #[inline]
    fn empty() -> Vector4<T> { Vector4::from_element(T::zero()) }

    #[inline]
    fn with_alpha(self, alpha: T) -> Vector4<T> {
        let XYZW { x, y, z, .. } = *self;

        Vector4::new(x, y, z, alpha)
    }

    #[inline]
    fn mul_alpha(self, alpha: T) -> Vector4<T> {
        let XYZW { x, y, z, w } = *self;

        Vector4::new(x, y, z, w * alpha)
    }

    #[inline]
    fn get_alpha(&self) -> T { self.w }
}

macro_rules! impl_vector_color_without_alpha {
    ($name:ident) => {
        impl<T> Color for $name<T> where T: Scalar + Num + Send + Sync + Default {
            type Alpha = ();

            #[inline]
            fn empty() -> $name<T> { $name::from_element(T::zero()) }

            #[inline(always)]
            fn with_alpha(self, _: ()) -> $name<T> { self }

            #[inline(always)]
            fn mul_alpha(self, _: ()) -> $name<T> { self }

            #[inline(always)]
            fn get_alpha(&self) -> () { () }
        }
    }
}

impl_vector_color_without_alpha!(Vector1);
impl_vector_color_without_alpha!(Vector2);
impl_vector_color_without_alpha!(Vector3);