//! Clipping implementation

use nalgebra::Vector4;
use nalgebra::coordinates::XYZW;

use ::render::{ClipVertex, Interpolate, PrimitiveStorage, PrimitiveRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClippingPlane {
    Left,
    Right,
    Top,
    Bottom,
    Near,
    Far,
}

/// All clipping planes in a constant array. Useful for iterating over all of them.
pub const ALL_CLIPPING_PLANES: [ClippingPlane; 6] = [
    ClippingPlane::Left,
    ClippingPlane::Right,
    ClippingPlane::Top,
    ClippingPlane::Bottom,
    ClippingPlane::Near,
    ClippingPlane::Far
];

impl ClippingPlane {
    /// Check if the clipping plane has the given clip-space point inside of it
    #[inline]
    pub fn has_inside<K>(self, v: &ClipVertex<K>) -> bool {
        let XYZW { x, y, z, w } = *v.position;

        match self {
            ClippingPlane::Left => { x >= -w }
            ClippingPlane::Right => { x <= w }
            ClippingPlane::Top => { y >= -w }
            ClippingPlane::Bottom => { y <= w }
            ClippingPlane::Near => { z >= 0.0 }
            ClippingPlane::Far => { z <= w }
        }
    }

    /// Find the intersection of a line and the clipping plane
    #[inline]
    pub fn intersect<K>(self, v1: &ClipVertex<K>, v2: &ClipVertex<K>) -> ClipVertex<K> where K: Interpolate {
        let XYZW { x: x1, y: y1, z: z1, w: w1 } = *v1.position;
        let XYZW { x: x2, y: y2, z: z2, w: w2 } = *v2.position;

        let (a, b) = match self {
            ClippingPlane::Left => (w1 + x1, w2 + x2),
            ClippingPlane::Right => (w1 - x1, w2 - x2),
            ClippingPlane::Top => (w1 + y1, w2 + y2),
            ClippingPlane::Bottom => (w1 - y1, w2 - y2),
            ClippingPlane::Near => (z1, z2),
            ClippingPlane::Far => (w1 - z1, w2 - z2),
        };

        let t = a / (a - b);

        ClipVertex {
            position: Interpolate::linear_interpolate(t, &v1.position, &v2.position),
            uniforms: Interpolate::linear_interpolate(t, &v1.uniforms, &v2.uniforms),
        }
    }
}