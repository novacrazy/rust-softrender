//! Pixel definition and operations

use std::fmt::Debug;

use nalgebra::Vector4;
use nalgebra::coordinates::XYZW;

/// Trait required to distinguish pixel type for use in the framebuffer and fragment shader
pub trait Pixel: Debug + Copy + Send + Sync + 'static {
    /// An empty pixel in which values can be accumulated into
    fn empty() -> Self;
    /// Copy the pixel, but with the given alpha channel value
    fn with_alpha(self, alpha: f32) -> Self;
    /// Copy the pixel, but multiply the alpha channel with the given value
    fn mul_alpha(self, alpha: f32) -> Self;
}

pub use self::formats::{RGBAu8Pixel, RGBAf32Pixel};

pub mod formats {
    use ::{Interpolate, linear_interpolate, barycentric_interpolate};

    use super::Pixel;

    /// 8-bit integer RGBA format
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct RGBAu8Pixel {
        /// Red Component
        pub r: u8,
        /// Green Component
        pub g: u8,
        /// Blue Component
        pub b: u8,
        /// Alpha Component
        pub a: u8,
    }

    /// 32-bit Floating point RGBA format
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

    impl Default for RGBAu8Pixel {
        fn default() -> RGBAu8Pixel {
            RGBAu8Pixel { r: 0, g: 0, b: 0, a: 255 }
        }
    }

    impl Default for RGBAf32Pixel {
        fn default() -> RGBAf32Pixel {
            RGBAf32Pixel { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
        }
    }

    impl From<RGBAf32Pixel> for RGBAu8Pixel {
        fn from(p: RGBAf32Pixel) -> RGBAu8Pixel {
            RGBAu8Pixel {
                r: (p.r.min(1.0) * 255.0).floor() as u8,
                g: (p.g.min(1.0) * 255.0).floor() as u8,
                b: (p.b.min(1.0) * 255.0).floor() as u8,
                a: (p.a.min(1.0) * 255.0).floor() as u8,
            }
        }
    }

    impl From<RGBAu8Pixel> for RGBAf32Pixel {
        fn from(p: RGBAu8Pixel) -> RGBAf32Pixel {
            RGBAf32Pixel {
                r: (p.r as f32 / 255.0),
                g: (p.g as f32 / 255.0),
                b: (p.b as f32 / 255.0),
                a: (p.a as f32 / 255.0),
            }
        }
    }

    impl Pixel for RGBAu8Pixel {
        fn empty() -> RGBAu8Pixel {
            RGBAu8Pixel { r: 0, g: 0, b: 0, a: 0 }
        }

        fn with_alpha(self, alpha: f32) -> RGBAu8Pixel {
            let alpha = (alpha.min(1.0) * 255.0).floor() as u8;

            RGBAu8Pixel { r: self.r, g: self.g, b: self.b, a: alpha }
        }

        fn mul_alpha(self, alpha: f32) -> RGBAu8Pixel {
            self.with_alpha((self.a as f32 / 255.0) * alpha)
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
    }

    impl Interpolate for RGBAu8Pixel {
        fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
            RGBAf32Pixel::barycentric_interpolate(u, &RGBAf32Pixel::from(*ux),
                                                  v, &RGBAf32Pixel::from(*vx),
                                                  w, &RGBAf32Pixel::from(*wx)).into()
        }

        fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
            RGBAf32Pixel::linear_interpolate(t,
                                             &RGBAf32Pixel::from(*x1),
                                             &RGBAf32Pixel::from(*x2)).into()
        }
    }

    impl Interpolate for RGBAf32Pixel {
        fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
            RGBAf32Pixel {
                r: barycentric_interpolate(u, ux.r, v, vx.r, w, wx.r),
                g: barycentric_interpolate(u, ux.g, v, vx.g, w, wx.g),
                b: barycentric_interpolate(u, ux.b, v, vx.b, w, wx.b),
                a: barycentric_interpolate(u, ux.a, v, vx.a, w, wx.a),
            }
        }

        fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
            RGBAf32Pixel {
                r: linear_interpolate(t, x1.r, x2.r),
                g: linear_interpolate(t, x1.g, x2.g),
                b: linear_interpolate(t, x1.b, x2.b),
                a: linear_interpolate(t, x1.a, x2.a),
            }
        }
    }
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
}