//! Storage structures

use ::numeric::FloatScalar;
use ::geometry::{ClipVertex, ScreenVertex};
use ::primitive::PrimitiveRef;

#[derive(Clone)]
pub ( in ::pipeline ) struct SeparablePrimitiveStorage<N: FloatScalar, K> {
    pub points: Vec<ClipVertex<N, K>>,
    pub lines: Vec<ClipVertex<N, K>>,
    pub tris: Vec<ClipVertex<N, K>>,
}

impl<N, K> Default for SeparablePrimitiveStorage<N, K> where N: FloatScalar {
    fn default() -> SeparablePrimitiveStorage<N, K> {
        SeparablePrimitiveStorage {
            points: Vec::new(),
            lines: Vec::new(),
            tris: Vec::new(),
        }
    }
}

impl<N, K> SeparablePrimitiveStorage<N, K> where N: FloatScalar {
    pub fn append(&mut self, other: &mut SeparablePrimitiveStorage<N, K>) {
        self.points.append(&mut other.points);
        self.lines.append(&mut other.lines);
        self.tris.append(&mut other.tris);
    }

    #[inline]
    pub fn push_point(&mut self, point: ClipVertex<N, K>) {
        self.points.push(point);
    }

    #[inline]
    pub fn push_line(&mut self, start: ClipVertex<N, K>, end: ClipVertex<N, K>) {
        self.lines.reserve(2);
        self.lines.push(start);
        self.lines.push(end);
    }

    #[inline]
    pub fn push_triangle(&mut self, a: ClipVertex<N, K>, b: ClipVertex<N, K>, c: ClipVertex<N, K>) {
        self.tris.reserve(3);
        self.tris.push(a);
        self.tris.push(b);
        self.tris.push(c);
    }
}

// Internal type for accumulating varying primitives in screen-space
#[derive(Clone)]
pub ( in ::pipeline ) struct SeparableScreenPrimitiveStorage<N: FloatScalar, K> {
    pub points: Vec<ScreenVertex<N, K>>,
    pub lines: Vec<ScreenVertex<N, K>>,
    pub tris: Vec<ScreenVertex<N, K>>,
}

impl<N, K> Default for SeparableScreenPrimitiveStorage<N, K> where N: FloatScalar {
    fn default() -> SeparableScreenPrimitiveStorage<N, K> {
        SeparableScreenPrimitiveStorage {
            points: Vec::new(),
            lines: Vec::new(),
            tris: Vec::new(),
        }
    }
}

/// Holds a reference to the internal storage structure for primitives
pub struct PrimitiveStorage<'s, N: FloatScalar, K: 's> {
    pub ( in ::pipeline ) inner: &'s mut SeparablePrimitiveStorage<N, K>,
}

impl<'s, N, K: 's> PrimitiveStorage<'s, N, K> where N: FloatScalar {
    /// Adds a point to the storage
    #[inline(always)]
    pub fn emit_point(&mut self, point: ClipVertex<N, K>) {
        self.inner.push_point(point);
    }

    /// Adds a line to the storage
    #[inline(always)]
    pub fn emit_line(&mut self, start: ClipVertex<N, K>, end: ClipVertex<N, K>) {
        self.inner.push_line(start, end);
    }

    /// Adds a triangle to the storage
    #[inline(always)]
    pub fn emit_triangle(&mut self, a: ClipVertex<N, K>, b: ClipVertex<N, K>, c: ClipVertex<N, K>) {
        self.inner.push_triangle(a, b, c)
    }

    #[inline]
    pub fn emit<'p>(&mut self, primitive: PrimitiveRef<'p, N, K>) where K: Clone {
        match primitive {
            PrimitiveRef::Point(point) => self.emit_point(point.clone()),
            PrimitiveRef::Line { start, end } => self.emit_line(start.clone(), end.clone()),
            PrimitiveRef::Triangle { a, b, c } => self.emit_triangle(a.clone(), b.clone(), c.clone()),
        }
    }
}