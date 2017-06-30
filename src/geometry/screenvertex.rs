use num_traits::Float;

use nalgebra::Vector4;

use ::numeric::FloatScalar;
use ::interpolate::Interpolate;

/// Defines a vertex and uniforms in screen-space, which is used in the fragment shader.
///
/// Clip-space vertices are transformed to screen-space after the vertex shader
/// stage but before the fragment shader stage.
#[derive(Debug, Clone)]
pub struct ScreenVertex<N: FloatScalar, K> {
    /// Screen-space vertex position. This is the position on screen of this vertex.
    ///
    /// Similar to `gl_FragCoord`
    pub position: Vector4<N>,
    /// Any custom data to be sent between shader stages, such as positions, normals, UV coordinates and whatever else
    /// you would usually put in uniforms to share between shader stages.
    pub uniforms: K,
}

impl<N, K> Interpolate for ScreenVertex<N, K> where N: FloatScalar,
                                                    K: Interpolate {
    #[inline]
    fn barycentric_interpolate<R: Float>(u: R, x1: &Self, v: R, x2: &Self, w: R, x3: &Self) -> Self {
        ScreenVertex {
            position: Interpolate::barycentric_interpolate(u, &x1.position, v, &x2.position, w, &x3.position),
            uniforms: Interpolate::barycentric_interpolate(u, &x1.uniforms, v, &x2.uniforms, w, &x3.uniforms),
        }
    }

    #[inline]
    fn linear_interpolate<R: Float>(t: R, x1: &Self, x2: &Self) -> Self {
        ScreenVertex {
            position: Interpolate::linear_interpolate(t, &x1.position, &x2.position),
            uniforms: Interpolate::linear_interpolate(t, &x1.uniforms, &x2.uniforms),
        }
    }
}