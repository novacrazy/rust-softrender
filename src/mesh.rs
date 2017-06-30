//! Generic mesh structure

use std::fmt::{Debug, Formatter, Result as FmtResult};

use nalgebra::Point3;

use ::behavior::ThreadSafeCopyable;
use ::numeric::FloatScalar;

/// A single vertex with a required position vector and any other vertex data
#[derive(Debug, Clone)]
pub struct SimpleVertex<N: FloatScalar, D> {
    /// Position of the vertex in object-space
    pub position: Point3<N>,
    /// Any data that goes alongside the required position, such as normals, UV coordinates, tangents, or whatever.
    ///
    /// This is separate because the position is required, but anything else is optional,
    /// so setting type `V` to `()` for no extra vertex data means no extra overhead.
    pub data: D,
}

pub trait Vertex: Send + Sync {
    type Scalar: ThreadSafeCopyable + FloatScalar;

    fn position(&self) -> Point3<Self::Scalar>;
}

impl<N, D> Vertex for SimpleVertex<N, D> where N: ThreadSafeCopyable + FloatScalar,
                                               D: Send + Sync {
    type Scalar = N;

    #[inline(always)]
    fn position(&self) -> Point3<N> { self.position }
}

/// Mesh structure with indexed vertices.
#[derive(Clone)]
pub struct Mesh<V: Vertex> {
    /// Vertex indices
    ///
    /// If you are unfamiliar with vertex indices, it's a way of re-using vertices for multiple primitives.
    ///
    /// For example (in 2D), for a rectangle made of two triangles, you would define the four points for each corner vertex:
    ///
    /// ```text
    /// vertex #: name         = (x,   y)
    /// 0:        bottom_left  = (0.0, 1.0)
    /// 1:        top_left     = (0.0, 1.0)
    /// 2:        bottom_right = (1.0, 0.0)
    /// 3:        top_right    = (1.0, 1.0)
    /// ```
    ///
    /// then you'd have your index list be something like:
    ///
    /// ```text
    /// [0, 1, 2, // bottom half triangle
    ///  1, 3, 2] // top half triangle
    /// ```
    ///
    /// Note that both of those triangles go in a clockwise direction from vertex to vertex.
    pub indices: Vec<usize>,
    /// Vertices with their vertex data
    pub vertices: Vec<V>,
}

impl<V> Debug for Mesh<V> where V: Vertex {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Mesh {{ vertices: {} }}", self.vertices.len())
    }
}