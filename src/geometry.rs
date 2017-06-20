//! Shader geometry structures

use nalgebra::Vector4;
use nalgebra::core::coordinates::XYZW;

use ::interpolate::Interpolate;

/// Defines face winding variations. These apply to screen-space vertices,
/// so imagine the vertices as they are viewed from the final image.
///
/// If all triangles of a mesh have the same face winding,
/// then triangles that are facing away from the screen can be skipped since they
/// will have the opposite winding order, since they are viewed from the back. This is known
/// as backface culling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceWinding {
    /// Clockwise face winding, where the vertices are like so:
    ///
    /// ```text
    ///                   1
    ///                  /|
    ///           A    /  |
    ///         /    /    |
    ///       /    /      | |
    ///     /    /        | |
    ///        /          | |
    ///      /            | V
    ///    /              |
    /// 3 *---------------* 2
    ///       <-------
    /// ```
    Clockwise,
    /// Counter-Clockwise face winding, where the vertices are like so:
    ///
    /// ```text
    ///                   1
    ///                  /|
    ///           /    /  |
    ///         /    /    |
    ///       /    /      | A
    ///     V    /        | |
    ///        /          | |
    ///      /            | |
    ///    /              |
    /// 3 *---------------* 2
    ///       ------->
    /// ```
    CounterClockwise
}

/// Defines a vertex and uniforms in clip-space, which is produced by the vertex shader stage.
#[derive(Debug, Clone)]
pub struct ClipVertex<K> {
    /// Clip-space vertex position. This isn't very useful to the user unless normalized.
    pub position: Vector4<f32>,
    /// Any custom data to be sent between shader stages, such as positions, normals, UV coordinates and whatever else
    /// you would usually put in uniforms to share between shader stages.
    pub uniforms: K,
}

impl<K> Interpolate for ClipVertex<K> where K: Interpolate {
    #[inline]
    fn barycentric_interpolate(u: f32, x1: &Self, v: f32, x2: &Self, w: f32, x3: &Self) -> Self {
        ClipVertex {
            position: Interpolate::barycentric_interpolate(u, &x1.position, v, &x2.position, w, &x3.position),
            uniforms: Interpolate::barycentric_interpolate(u, &x1.uniforms, v, &x2.uniforms, w, &x3.uniforms),
        }
    }

    #[inline]
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
        ClipVertex {
            position: Interpolate::linear_interpolate(t, &x1.position, &x2.position),
            uniforms: Interpolate::linear_interpolate(t, &x1.uniforms, &x2.uniforms),
        }
    }
}

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

impl<K> ClipVertex<K> where K: Send + Sync {
    /// Creates a new `ClipVertex` from the given clip-space position and uniforms
    #[inline(always)]
    pub fn new(position: Vector4<f32>, uniforms: K) -> ClipVertex<K> {
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
    pub fn normalize(self, viewport: (f32, f32)) -> ScreenVertex<K> {
        ScreenVertex {
            position: {
                let (width, height) = viewport;

                let XYZW { x, y, z, w } = *self.position;

                Vector4::new(
                    (1.0 + x / w) * width / 2.0,
                    (1.0 - y / w) * height / 2.0,
                    z / w,
                    1.0 / w
                )
            },
            uniforms: self.uniforms,
        }
    }
}