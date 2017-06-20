//! Color Buffer attachment definitions

pub mod blend;

/// Defines a Color buffer attachment
pub trait Color: super::Attachment {
    /// An empty pixel in which values can be accumulated into
    fn empty() -> Self;
    /// Copy the pixel, but with the given alpha channel value
    fn with_alpha(self, alpha: f32) -> Self;
    /// Copy the pixel, but multiply the alpha channel with the given value
    fn mul_alpha(self, alpha: f32) -> Self;
}

impl Color for () {
    fn empty() -> () { () }
    fn with_alpha(self, _: f32) -> () { () }
    fn mul_alpha(self, _: f32) -> () { () }
}

pub mod predefined;