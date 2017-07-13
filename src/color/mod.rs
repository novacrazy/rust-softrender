//! Color type definitions, for both framebuffer usage and general usage

use num_traits::NumCast;

use ::behavior::ThreadSafeCopyable;
use ::numeric::FloatScalar;

pub mod blend;
pub mod helper;

pub use self::helper::AlphaMultiply;

pub trait ColorAlpha: ThreadSafeCopyable + Default {
    fn from_scalar<N: FloatScalar>(n: N) -> Self;
}

impl ColorAlpha for () {
    #[inline(always)]
    fn from_scalar<N: FloatScalar>(_: N) -> () { () }
}

macro_rules! impl_color_alpha {
    ($($t:ty),+) => {
        $(
            impl ColorAlpha for $t {
                #[inline(always)]
                fn from_scalar<N: FloatScalar>(n: N) -> $t {
                    <$t as NumCast>::from(n).expect("Invalid Cast")
                }
            }
        )+
    }
}

impl_color_alpha!(i8, i16, i32, i64, u8, u16, u32, u64, isize, usize, f32, f64);

/// Defines a Color buffer attachment
pub trait Color: ThreadSafeCopyable {
    type Alpha: ColorAlpha;

    /// An empty pixel in which values can be accumulated into
    fn empty() -> Self;
    /// Copy the pixel, but with the given alpha channel value
    fn with_alpha(self, alpha: Self::Alpha) -> Self;
    /// Copy the pixel, but multiply the alpha channel with the given value
    fn mul_alpha(self, alpha: Self::Alpha) -> Self;
    /// Get the alpha of the color
    fn get_alpha(&self) -> Self::Alpha;
}

impl Color for () {
    type Alpha = ();

    fn empty() -> () { () }
    fn with_alpha(self, _: Self::Alpha) -> () { () }
    fn mul_alpha(self, _: Self::Alpha) -> () { () }
    fn get_alpha(&self) -> Self::Alpha { () }
}

pub mod predefined;

#[doc(hidden)]
pub fn __assert_color<C: Color>() {}
