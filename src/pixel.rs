//! Pixel definition and operations

/// Trait required to distinguish pixel type for use in the framebuffer and fragment shader
pub trait Pixel: Clone + Copy + Send + Sync {
    fn empty() -> Self;
}

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

impl Default for RGBAf32Pixel {
    fn default() -> RGBAf32Pixel {
        RGBAf32Pixel { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }
}

impl Pixel for RGBAf32Pixel {
    fn empty() -> RGBAf32Pixel {
        RGBAf32Pixel { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
}