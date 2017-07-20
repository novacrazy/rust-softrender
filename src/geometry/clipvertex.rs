use num_traits::Float;

use nalgebra::{Vector4, Matrix4};
use nalgebra::core::coordinates::XYZW;

use ::numeric::FloatScalar;
use ::interpolate::Interpolate;

use super::{Dimensions, Coordinate, ScreenVertex};

/// Defines a vertex and uniforms in clip-space, which is produced by the vertex shader stage.
#[derive(Debug, Clone)]
pub struct ClipVertex<N: FloatScalar, K> {
    /// Clip-space vertex position. This isn't very useful to the user unless normalized.
    pub position: Vector4<N>,
    /// Any custom data to be sent between shader stages, such as positions, normals, UV coordinates and whatever else
    /// you would usually put in uniforms to share between shader stages.
    pub uniforms: K,
}

impl<N, K> Interpolate for ClipVertex<N, K> where N: FloatScalar, K: Interpolate {
    #[inline]
    fn barycentric_interpolate<R: Float>(u: R, x1: &Self, v: R, x2: &Self, w: R, x3: &Self) -> Self {
        ClipVertex {
            position: Interpolate::barycentric_interpolate(u, &x1.position, v, &x2.position, w, &x3.position),
            uniforms: Interpolate::barycentric_interpolate(u, &x1.uniforms, v, &x2.uniforms, w, &x3.uniforms),
        }
    }

    #[inline]
    fn linear_interpolate<R: Float>(t: R, x1: &Self, x2: &Self) -> Self {
        ClipVertex {
            position: Interpolate::linear_interpolate(t, &x1.position, &x2.position),
            uniforms: Interpolate::linear_interpolate(t, &x1.uniforms, &x2.uniforms),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Viewport<N> where N: FloatScalar {
    pub x: N,
    pub y: N,
    pub width: N,
    pub height: N,
    pub near: N,
    pub far: N
}

impl<N> Viewport<N> where N: FloatScalar {
    pub fn new(dimensions: Dimensions, offset: Coordinate, near: N, far: N) -> Viewport<N> {
        Viewport {
            x: N::from(offset.x).unwrap(),
            y: N::from(offset.y).unwrap(),
            width: N::from(dimensions.width).unwrap(),
            height: N::from(dimensions.height).unwrap(),
            near,
            far
        }
    }

    pub fn aspect_ratio(&self) -> N {
        self.width / self.height
    }
}

impl<N, K> ClipVertex<N, K> where N: FloatScalar,
                                  K: Send + Sync {
    /// Creates a new `ClipVertex` from the given clip-space position and uniforms
    #[inline(always)]
    pub fn new(position: Vector4<N>, uniforms: K) -> ClipVertex<N, K> {
        ClipVertex { position: position, uniforms: uniforms }
    }

    /// Normalizes the clip-space vertex coordinates to screen-space using the given viewport.
    ///
    /// This assumes a viewport in the shape of:
    ///
    /// ```text
    /// 0,0-----------------x
    ///  |                  |
    ///  |                  |
    ///  |                  |
    ///  |                  |
    ///  |                  |
    ///  y-----------------x,y
    /// ```
    ///
    /// where the y-axis is flipped.
    pub fn normalize(self, viewport: Viewport<N>) -> ScreenVertex<N, K> {
        ScreenVertex {
            position: {
                let XYZW { x, y, z, w } = *self.position;

                let Viewport {
                    x: left, y: bottom,
                    width, height,
                    near, far
                } = viewport;

                let right = left + width;
                let top = bottom + height;

                macro_rules! n {
                    ($v:expr) => {N::from($v).unwrap()}
                }

                let viewport_matrix = Matrix4::new(
                    (right - left) / n!(2.0), N::zero(), N::zero(), (right + left) / n!(2.0),
                    N::zero(), (top - bottom) / n!(-2.0), N::zero(), (top + bottom) / n!(2.0),
                    N::zero(), N::zero(), (far - near) / n!(-2.0), (far + near) / n!(-2.0),
                    N::zero(), N::zero(), N::zero(), N::one(),
                );

                let mut screen = viewport_matrix * Vector4::new(
                    x / w,
                    y / w,
                    z / w,
                    N::one()
                );

                screen.w = N::one() / w;

                screen
            },
            uniforms: self.uniforms,
        }
    }
}