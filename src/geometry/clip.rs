//! Clipping planes

use num_traits::Zero;

use nalgebra::coordinates::XYZW;

use ::numeric::FloatScalar;
use ::geometry::ClipVertex;
use ::interpolate::Interpolate;

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
    pub fn has_inside<N: FloatScalar, K>(self, v: &ClipVertex<N, K>) -> bool {
        let XYZW { x, y, z, w } = *v.position;

        match self {
            ClippingPlane::Left => { x >= -w }
            ClippingPlane::Right => { x <= w }
            ClippingPlane::Top => { y >= -w }
            ClippingPlane::Bottom => { y <= w }
            ClippingPlane::Near => { z >= Zero::zero() }
            ClippingPlane::Far => { z <= w }
        }
    }

    /// Find the intersection of a line and the clipping plane
    #[inline]
    pub fn intersect<N: FloatScalar, K>(self, v1: &ClipVertex<N, K>, v2: &ClipVertex<N, K>) -> ClipVertex<N, K> where K: Interpolate {
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

        Interpolate::linear_interpolate(t, &v1, &v2)
    }
}