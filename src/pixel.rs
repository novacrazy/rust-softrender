//! Pixel definition and operations

/// Trait required to distinguish pixel type for use in the framebuffer and fragment shader
pub trait Pixel: Send + Sync {}

/// The most dead simple f32 pixel representation
#[derive(Debug, Clone, Copy)]
#[repr(C)] //to prevent field reordering
pub struct RGBAf32Pixel {
    /// Red Component
    pub r: f32,
    /// Green Component
    pub g: f32,
    /// Blue Component
    pub b: f32,
    /// Alpha Component
    pub a: f32,
}

impl Pixel for RGBAf32Pixel {}