//! Pixel definition and operations

use std::fmt::Debug;

use nalgebra::Vector4;
use nalgebra::coordinates::XYZW;

/// Trait required to distinguish pixel type for use in the framebuffer and fragment shader
pub trait Pixel: Debug + Clone + Copy + Send + Sync + 'static {
    /// An empty pixel in which values can be accumulated into
    fn empty() -> Self;
    /// Copy the pixel, but with the given alpha channel value
    fn with_alpha(self, alpha: f32) -> Self;
    /// Copy the pixel, but multiply the alpha channel with the given value
    fn mul_alpha(self, alpha: f32) -> Self;

    fn alpha(&self) -> f32;
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

    fn with_alpha(self, alpha: f32) -> RGBAf32Pixel {
        RGBAf32Pixel { r: self.r, g: self.g, b: self.b, a: alpha }
    }

    fn mul_alpha(self, alpha: f32) -> RGBAf32Pixel {
        RGBAf32Pixel { r: self.r, g: self.g, b: self.b, a: self.a * alpha }
    }

    #[inline(always)]
    fn alpha(&self) -> f32 { self.a }
}

impl Pixel for Vector4<f32> {
    fn empty() -> Vector4<f32> {
        Vector4::new(0.0, 0.0, 0.0, 0.0)
    }

    fn with_alpha(self, alpha: f32) -> Vector4<f32> {
        let XYZW { x, y, z, .. } = *self;

        Vector4::new(x, y, z, alpha)
    }

    fn mul_alpha(self, alpha: f32) -> Vector4<f32> {
        let XYZW { x, y, z, w } = *self;

        Vector4::new(x, y, z, w * alpha)
    }

    #[inline(always)]
    fn alpha(&self) -> f32 { self.w }
}