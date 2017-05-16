use ::render::ClipVertex;

/// Defines the kinds of primitives that can be rendered by themselves.
pub trait Primitive {
    /// Get's the number of vertices for the given primitive type
    fn num_vertices() -> usize;

    /// Creates a `PrimitiveRef` from some vertices
    ///
    /// This is used internally.
    fn create_ref_from_vertices<'p, K>(vertices: &'p [ClipVertex<K>]) -> PrimitiveRef<'p, K>;

    /// Creates a `PrimitiveRef` from some indexed vertices.
    ///
    /// This are used internally.
    fn create_ref_from_indexed_vertices<'p, K>(vertices: &'p [ClipVertex<K>], indices: &[u32]) -> PrimitiveRef<'p, K>;
}

/// Holds references to primitive vertices for each primitive type
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveRef<'p, K: 'p> {
    Point(&'p ClipVertex<K>),
    Line {
        start: &'p ClipVertex<K>,
        end: &'p ClipVertex<K>,
    },
    Triangle {
        a: &'p ClipVertex<K>,
        b: &'p ClipVertex<K>,
        c: &'p ClipVertex<K>,
    }
}

/// Individual points
#[derive(Debug, Clone, Copy)]
pub struct Point;

/// Lines between two vertices
#[derive(Debug, Clone, Copy)]
pub struct Line;

/// Triangles between three vertices
#[derive(Debug, Clone, Copy)]
pub struct Triangle;

impl Primitive for Point {
    fn num_vertices() -> usize { 1 }

    fn create_ref_from_vertices<'p, K>(vertices: &'p [ClipVertex<K>]) -> PrimitiveRef<'p, K> {
        assert_eq!(vertices.len(), Self::num_vertices());

        PrimitiveRef::Point(&vertices[0])
    }

    fn create_ref_from_indexed_vertices<'p, K>(vertices: &'p [ClipVertex<K>], indices: &[u32]) -> PrimitiveRef<'p, K> {
        assert_eq!(indices.len(), Self::num_vertices());

        PrimitiveRef::Point(&vertices[indices[0] as usize])
    }
}

impl Primitive for Line {
    fn num_vertices() -> usize { 2 }

    fn create_ref_from_vertices<'p, K>(vertices: &'p [ClipVertex<K>]) -> PrimitiveRef<'p, K> {
        assert_eq!(vertices.len(), Self::num_vertices());

        PrimitiveRef::Line { start: &vertices[0], end: &vertices[1] }
    }

    fn create_ref_from_indexed_vertices<'p, K>(vertices: &'p [ClipVertex<K>], indices: &[u32]) -> PrimitiveRef<'p, K> {
        assert_eq!(indices.len(), Self::num_vertices());

        PrimitiveRef::Line {
            start: &vertices[indices[0] as usize],
            end: &vertices[indices[1] as usize],
        }
    }
}

impl Primitive for Triangle {
    fn num_vertices() -> usize { 3 }

    fn create_ref_from_vertices<'p, K>(vertices: &'p [ClipVertex<K>]) -> PrimitiveRef<'p, K> {
        assert_eq!(vertices.len(), Self::num_vertices());

        PrimitiveRef::Triangle {
            a: &vertices[0],
            b: &vertices[1],
            c: &vertices[2],
        }
    }

    fn create_ref_from_indexed_vertices<'p, K>(vertices: &'p [ClipVertex<K>], indices: &[u32]) -> PrimitiveRef<'p, K> {
        assert_eq!(indices.len(), Self::num_vertices());

        PrimitiveRef::Triangle {
            a: &vertices[indices[0] as usize],
            b: &vertices[indices[1] as usize],
            c: &vertices[indices[2] as usize],
        }
    }
}