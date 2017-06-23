//! Color type definitions, for both framebuffer usage and general usage

use ::behavior::ThreadSafeCopyable;

pub mod blend;

/// Defines a Color buffer attachment
pub trait Color: ThreadSafeCopyable {
    type Alpha: ThreadSafeCopyable + Default;

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
