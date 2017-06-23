use nalgebra::Vector4;

use ::interpolate::Interpolate;

/// Defines a vertex and uniforms in screen-space, which is used in the fragment shader.
///
/// Clip-space vertices are transformed to screen-space after the vertex shader
/// stage but before the fragment shader stage.
#[derive(Debug, Clone)]
pub struct ScreenVertex<K> {
    /// Screen-space vertex position. This is the position on screen of this vertex.
    ///
    /// Similar to `gl_FragCoord`
    pub position: Vector4<f32>,
    /// Any custom data to be sent between shader stages, such as positions, normals, UV coordinates and whatever else
    /// you would usually put in uniforms to share between shader stages.
    pub uniforms: K,
}

impl<K> Interpolate for ScreenVertex<K> where K: Interpolate {
    #[inline]
    fn barycentric_interpolate(u: f32, x1: &Self, v: f32, x2: &Self, w: f32, x3: &Self) -> Self {
        ScreenVertex {
            position: Interpolate::barycentric_interpolate(u, &x1.position, v, &x2.position, w, &x3.position),
            uniforms: Interpolate::barycentric_interpolate(u, &x1.uniforms, v, &x2.uniforms, w, &x3.uniforms),
        }
    }

    #[inline]
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
        ScreenVertex {
            position: Interpolate::linear_interpolate(t, &x1.position, &x2.position),
            uniforms: Interpolate::linear_interpolate(t, &x1.uniforms, &x2.uniforms),
        }
    }
}