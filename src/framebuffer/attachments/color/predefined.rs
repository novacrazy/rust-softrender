use num_traits::Zero;

use nalgebra::Vector4;
use nalgebra::coordinates::XYZW;

use super::Color;

impl Color for Vector4<f32> {
    #[inline]
    fn empty() -> Vector4<f32> { Vector4::zero() }

    #[inline]
    fn with_alpha(self, alpha: f32) -> Vector4<f32> {
        let XYZW { x, y, z, .. } = *self;

        Vector4::new(x, y, z, alpha)
    }

    #[inline]
    fn mul_alpha(self, alpha: f32) -> Vector4<f32> {
        let XYZW { x, y, z, w } = *self;

        Vector4::new(x, y, z, w * alpha)
    }
}