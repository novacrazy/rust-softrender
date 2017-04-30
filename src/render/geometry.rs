use nalgebra::Vector4;
use nalgebra::core::coordinates::XYZW;

use ::render::Barycentric;

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
pub struct ClipVertex<K> where K: Send + Sync + Barycentric {
    /// Clip-space vertex position. This isn't very useful to the user unless normalized.
    pub position: Vector4<f32>,
    /// Any custom data to be sent between shader stages, such as positions, normals, UV coordinates and whatever else
    /// you would usually put in uniforms to share between shader stages.
    pub uniforms: K,
}

/// Defines a vertex and uniforms in screen-space, which is used in the fragment shader.
///
/// Clip-space vertices are transformed to screen-space after the vertex shader
/// stage but before the fragment shader stage.
#[derive(Debug, Clone)]
pub struct ScreenVertex<K> where K: Send + Sync + Barycentric {
    /// Screen-space vertex position. This is the position on screen of this vertex.
    ///
    /// Similar to `gl_FragCoord`
    pub position: Vector4<f32>,
    /// Any custom data to be sent between shader stages, such as positions, normals, UV coordinates and whatever else
    /// you would usually put in uniforms to share between shader stages.
    pub uniforms: K,
}

impl<K> ClipVertex<K> where K: Send + Sync + Barycentric {
    #[inline(always)]
    pub fn new(position: Vector4<f32>, uniforms: K) -> ClipVertex<K> {
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
    pub fn normalize(self, viewport: (f32, f32)) -> ScreenVertex<K> {
        ScreenVertex {
            position: {
                let (width, height) = viewport;

                let XYZW { x, y, z, w } = *self.position;

                Vector4::new(
                    (x / w + 1.0) * (width / 2.0),
                    (1.0 - y / w) * (height / 2.0),
                    z / w,
                    1.0 / w
                )
            },
            uniforms: self.uniforms,
        }
    }
}

impl<K> ScreenVertex<K> where K: Send + Sync + Barycentric {
    #[inline(always)]
    pub fn new(position: Vector4<f32>, uniforms: K) -> ScreenVertex<K> {
        ScreenVertex { position: position, uniforms: uniforms }
    }
}