use num_traits::{One, Zero};

use ::numeric::FloatScalar;

#[inline]
pub fn liang_barsky_iterative<T: FloatScalar>(start: (T, T), end: (T, T), bounds: ((T, T), (T, T))) -> Option<((T, T), (T, T))> {
    let ((xmin, ymin), (xmax, ymax)) = bounds;

    let (mut x1, mut y1) = start;
    let (mut x2, mut y2) = end;

    let mut t0 = Zero::zero();
    let mut t1 = One::one();

    let dx = x2 - x1;
    let dy = y2 - y1;

    for edge in 0..4 {
        let (p, q) = match edge {
            0 => (-dx, -(xmin - x1)),
            1 => (dx, (xmax - x1)),
            2 => (-dy, -(ymin - y1)),
            3 => (dy, (ymax - y1)),
            _ => unreachable!()
        };

        if p.is_zero() && q < Zero::zero() {
            return None;
        } else {
            let r = q / p;

            if p < Zero::zero() {
                if r > t1 {
                    return None;
                } else if r > t0 {
                    t0 = r;
                }
            } else if p > Zero::zero() {
                if r < t0 {
                    return None;
                } else if r < t1 {
                    t1 = r;
                }
            }
        }
    }

    let x1clip = x1 + t0 * dx;
    let y1clip = y1 + t0 * dy;
    let x2clip = x1 + t1 * dx;
    let y2clip = y1 + t1 * dy;

    Some(((x1clip, y1clip), (x2clip, y2clip)))
}
