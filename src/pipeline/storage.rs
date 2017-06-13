use ::{ClipVertex, ScreenVertex, PrimitiveRef};

// Internal type for accumulating varying primitives
#[derive(Clone)]
pub ( crate ) struct SeparablePrimitiveStorage<K> {
    pub points: Vec<ClipVertex<K>>,
    pub lines: Vec<ClipVertex<K>>,
    pub tris: Vec<ClipVertex<K>>,
}

impl<K> Default for SeparablePrimitiveStorage<K> {
    fn default() -> SeparablePrimitiveStorage<K> {
        SeparablePrimitiveStorage {
            points: Vec::new(),
            lines: Vec::new(),
            tris: Vec::new(),
        }
    }
}

// Internal type for accumulating varying primitives in screen-space
#[derive(Clone)]
pub ( crate ) struct SeparableScreenPrimitiveStorage<K> {
    pub points: Vec<ScreenVertex<K>>,
    pub lines: Vec<ScreenVertex<K>>,
    pub tris: Vec<ScreenVertex<K>>,
}

impl<K> Default for SeparableScreenPrimitiveStorage<K> {
    fn default() -> SeparableScreenPrimitiveStorage<K> {
        SeparableScreenPrimitiveStorage {
            points: Vec::new(),
            lines: Vec::new(),
            tris: Vec::new(),
        }
    }
}

impl<K> SeparablePrimitiveStorage<K> {
    pub fn append(&mut self, other: &mut SeparablePrimitiveStorage<K>) {
        self.points.append(&mut other.points);
        self.lines.append(&mut other.lines);
        self.tris.append(&mut other.tris);
    }

    #[inline]
    pub fn push_point(&mut self, point: ClipVertex<K>) {
        self.points.push(point);
    }

    #[inline]
    pub fn push_line(&mut self, start: ClipVertex<K>, end: ClipVertex<K>) {
        self.lines.reserve(2);
        self.lines.push(start);
        self.lines.push(end);
    }

    #[inline]
    pub fn push_triangle(&mut self, a: ClipVertex<K>, b: ClipVertex<K>, c: ClipVertex<K>) {
        self.tris.reserve(3);
        self.tris.push(a);
        self.tris.push(b);
        self.tris.push(c);
    }
}

/// Holds a reference to the internal storage structure for primitives
pub struct PrimitiveStorage<'s, K> where K: 's {
    inner: &'s mut SeparablePrimitiveStorage<K>,
}

impl<'s, K> PrimitiveStorage<'s, K> where K: 's {
    /// Adds a point to the storage
    #[inline(always)]
    pub fn emit_point(&mut self, point: ClipVertex<K>) {
        self.inner.push_point(point);
    }

    /// Adds a line to the storage
    #[inline(always)]
    pub fn emit_line(&mut self, start: ClipVertex<K>, end: ClipVertex<K>) {
        self.inner.push_line(start, end);
    }

    /// Adds a triangle to the storage
    #[inline(always)]
    pub fn emit_triangle(&mut self, a: ClipVertex<K>, b: ClipVertex<K>, c: ClipVertex<K>) {
        self.inner.push_triangle(a, b, c)
    }

    pub fn re_emit<'p>(&mut self, primitive: PrimitiveRef<'p, K>) where K: Clone {
        match primitive {
            PrimitiveRef::Point(point) => self.emit_point(point.clone()),
            PrimitiveRef::Line { start, end } => self.emit_line(start.clone(), end.clone()),
            PrimitiveRef::Triangle { a, b, c } => self.emit_triangle(a.clone(), b.clone(), c.clone()),
        }
    }
}