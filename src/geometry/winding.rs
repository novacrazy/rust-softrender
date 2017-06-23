//! Polygon face winding definitions

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