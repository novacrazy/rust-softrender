use num_traits::Float;

use nalgebra::Vector4;
use nalgebra::core::coordinates::XYZW;

use ::numeric::FloatScalar;
use ::interpolate::Interpolate;

use super::ScreenVertex;

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

impl<N, K> ClipVertex<N, K> where N: FloatScalar,
                                  K: Send + Sync {
    /// Creates a new `ClipVertex` from the given clip-space position and uniforms
    #[inline(always)]
    pub fn new(position: Vector4<N>, uniforms: K) -> ClipVertex<N, K> {
        ClipVertex { position: position, uniforms: uniforms }
    }

    /// TODO: Move this to shader stage
    ///
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
    pub fn normalize(self, viewport: (N, N)) -> ScreenVertex<N, K> {
        ScreenVertex {
            position: {
                let (width, height) = viewport;

                let XYZW { x, y, z, w } = *self.position;

                Vector4::new(
                    (N::one() + x / w) * width / N::from(2.0).unwrap(),
                    (N::one() - y / w) * height / N::from(2.0).unwrap(),
                    -z / w,
                    N::one() / w
                )
            },
            uniforms: self.uniforms,
        }
    }
}