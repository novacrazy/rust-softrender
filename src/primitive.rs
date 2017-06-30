//! Primitive type-ids and reference enums

use ::numeric::FloatScalar;
use ::geometry::ClipVertex;

/// Defines the kinds of primitives that can be rendered by themselves.
pub trait Primitive {
    /// Get's the number of vertices for the given primitive type
    fn num_vertices() -> usize;

    /// Creates a `PrimitiveRef` from some vertices
    ///
    /// This is used internally.
    fn create_ref_from_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>]) -> PrimitiveRef<'p, N, K>;

    /// Creates a 'PrimitiveMut` from some vertices
    ///
    /// This is used internally.
    fn create_mut_from_vertices<'p, N: FloatScalar, K>(vertices: &'p mut [ClipVertex<N, K>]) -> PrimitiveMut<'p, N, K>;

    /// Creates a `PrimitiveRef` from some indexed vertices.
    ///
    /// This are used internally.
    fn create_ref_from_indexed_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>], indices: &[usize]) -> PrimitiveRef<'p, N, K>;
}

/// Holds references to primitive vertices for each primitive type
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveRef<'p, N: FloatScalar, K: 'p> {
    Point(&'p ClipVertex<N, K>),
    Line {
        start: &'p ClipVertex<N, K>,
        end: &'p ClipVertex<N, K>,
    },
    Triangle {
        a: &'p ClipVertex<N, K>,
        b: &'p ClipVertex<N, K>,
        c: &'p ClipVertex<N, K>,
    }
}

/// Holds mutable references to primitive vertices for each primitive type
#[derive(Debug)]
pub enum PrimitiveMut<'p, N: FloatScalar, K: 'p> {
    Point(&'p mut ClipVertex<N, K>),
    Line {
        start: &'p mut ClipVertex<N, K>,
        end: &'p mut ClipVertex<N, K>,
    },
    Triangle {
        a: &'p mut ClipVertex<N, K>,
        b: &'p mut ClipVertex<N, K>,
        c: &'p mut ClipVertex<N, K>,
    }
}

/// Individual points
#[derive(Debug, Clone, Copy, Default)]
pub struct Point;

/// Lines between two vertices
#[derive(Debug, Clone, Copy, Default)]
pub struct Line;

/// Triangles between three vertices
#[derive(Debug, Clone, Copy, Default)]
pub struct Triangle;

impl Primitive for Point {
    #[inline(always)]
    fn num_vertices() -> usize { 1 }

    fn create_ref_from_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>]) -> PrimitiveRef<'p, N, K> {
        debug_assert_eq!(vertices.len(), Self::num_vertices());

        PrimitiveRef::Point(&vertices[0])
    }

    fn create_mut_from_vertices<'p, N: FloatScalar, K>(vertices: &'p mut [ClipVertex<N, K>]) -> PrimitiveMut<'p, N, K> {
        debug_assert_eq!(vertices.len(), Self::num_vertices());

        PrimitiveMut::Point(&mut vertices[0])
    }

    fn create_ref_from_indexed_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>], indices: &[usize]) -> PrimitiveRef<'p, N, K> {
        debug_assert_eq!(indices.len(), Self::num_vertices());

        PrimitiveRef::Point(&vertices[indices[0] as usize])
    }
}

impl Primitive for Line {
    #[inline(always)]
    fn num_vertices() -> usize { 2 }

    fn create_ref_from_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>]) -> PrimitiveRef<'p, N, K> {
        debug_assert_eq!(vertices.len(), Self::num_vertices());

        PrimitiveRef::Line { start: &vertices[0], end: &vertices[1] }
    }

    fn create_mut_from_vertices<'p, N: FloatScalar, K>(vertices: &'p mut [ClipVertex<N, K>]) -> PrimitiveMut<'p, N, K> {
        debug_assert_eq!(vertices.len(), Self::num_vertices());

        let (mut start, mut end) = vertices.split_at_mut(1);

        PrimitiveMut::Line { start: &mut start[0], end: &mut end[0] }
    }

    fn create_ref_from_indexed_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>], indices: &[usize]) -> PrimitiveRef<'p, N, K> {
        debug_assert_eq!(indices.len(), Self::num_vertices());

        PrimitiveRef::Line {
            start: &vertices[indices[0] as usize],
            end: &vertices[indices[1] as usize],
        }
    }
}

impl Primitive for Triangle {
    #[inline(always)]
    fn num_vertices() -> usize { 3 }

    fn create_ref_from_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>]) -> PrimitiveRef<'p, N, K> {
        debug_assert_eq!(vertices.len(), Self::num_vertices());

        PrimitiveRef::Triangle {
            a: &vertices[0],
            b: &vertices[1],
            c: &vertices[2],
        }
    }

    fn create_mut_from_vertices<'p, N: FloatScalar, K>(vertices: &'p mut [ClipVertex<N, K>]) -> PrimitiveMut<'p, N, K> {
        debug_assert_eq!(vertices.len(), Self::num_vertices());

        let (mut a, mut bc) = vertices.split_at_mut(1);
        let (mut b, mut c) = bc.split_at_mut(1);

        PrimitiveMut::Triangle { a: &mut a[0], b: &mut b[0], c: &mut c[0] }
    }

    fn create_ref_from_indexed_vertices<'p, N: FloatScalar, K>(vertices: &'p [ClipVertex<N, K>], indices: &[usize]) -> PrimitiveRef<'p, N, K> {
        debug_assert_eq!(indices.len(), Self::num_vertices());

        PrimitiveRef::Triangle {
            a: &vertices[indices[0] as usize],
            b: &vertices[indices[1] as usize],
            c: &vertices[indices[2] as usize],
        }
    }
}