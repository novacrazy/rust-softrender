//! Interpolation utilities

use std::ops::{Add, Mul};

/// Describes a type that can be interpolated with barycentric coordinates.
///
/// This is required for any rasterization to occur.
///
/// See [This document](https://classes.soe.ucsc.edu/cmps160/Fall10/resources/barycentricInterpolation.pdf) for more information.
///
/// This trait can be implemented automatically for most uniforms by using the [`declare_uniforms!`](../../macro.declare_uniforms.html) macro,
/// which for any collection of uniforms for which `Interpolate` is implemented, will delegate `Interpolate::barycentric_interpolate` to each member.
pub trait Interpolate {
    /// Interpolate the three values with their corresponding barycentric coordinate weight
    fn barycentric_interpolate(u: f32, x1: &Self, v: f32, x2: &Self, w: f32, x3: &Self) -> Self;

    /// Simple linear interpolation
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self;
}

/// Convenience method for interpolating three values with barycentric coordinates.
#[inline]
pub fn barycentric_interpolate<T>(u: f32, ux: T, v: f32, vx: T, w: f32, wx: T) -> T where T: Add<Output=T> + Mul<f32, Output=T> {
    ux * u + vx * v + wx * w
}

/// Convenience method for linearly interpolating two values
#[inline]
pub fn linear_interpolate<T>(t: f32, x1: T, x2: T) -> T where T: Add<Output=T>, T: Mul<f32, Output=T> {
    x1 * (1.0 - t) + x2 * t
}

impl Interpolate for () {
    #[inline(always)]
    fn barycentric_interpolate(_: f32, _: &Self, _: f32, _: &Self, _: f32, _: &Self) -> Self { () }

    #[inline(always)]
    fn linear_interpolate(_: f32, _: &Self, _: &Self) -> Self { () }
}

impl Interpolate for f32 {
    #[inline(always)]
    fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
        ux * u + vx * v + wx * w
    }

    #[inline(always)]
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
        (1.0 - t) * x1 + t * x2
    }
}

use alga::general::Real;

use nalgebra::{Scalar, Matrix};
use nalgebra::{PointBase, QuaternionBase, RotationBase, TranslationBase};
use nalgebra::dimension::{DimName, U1, U2, U3, U4, U5, U6};
use nalgebra::allocator::OwnedAllocator;
use nalgebra::storage::{Storage, OwnedStorage};

impl<N, D, S> Interpolate for PointBase<N, D, S> where N: Scalar,
                                                       D: DimName,
                                                       S: Storage<N, D, U1>,
                                                       Matrix<N, D, U1, S>: Interpolate {
    #[inline]
    fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
        PointBase {
            coords: Interpolate::barycentric_interpolate(u, &ux.coords,
                                                         v, &vx.coords,
                                                         w, &wx.coords)
        }
    }

    #[inline]
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
        PointBase {
            coords: Interpolate::linear_interpolate(t, &x1.coords, &x2.coords)
        }
    }
}

impl<N, S> Interpolate for QuaternionBase<N, S> where N: Real,
                                                      S: Storage<N, U4, U1>,
                                                      Matrix<N, U4, U1, S>: Interpolate {
    #[inline]
    fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
        QuaternionBase {
            coords: Interpolate::barycentric_interpolate(u, &ux.coords,
                                                         v, &vx.coords,
                                                         w, &wx.coords)
        }
    }

    #[inline]
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
        QuaternionBase {
            coords: Interpolate::linear_interpolate(t, &x1.coords, &x2.coords)
        }
    }
}

impl<N, D: DimName, S> Interpolate for TranslationBase<N, D, S> where N: Scalar,
                                                                      Matrix<N, D, U1, S>: Interpolate {
    #[inline]
    fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
        TranslationBase {
            vector: Interpolate::barycentric_interpolate(u, &ux.vector,
                                                         v, &vx.vector,
                                                         w, &wx.vector)
        }
    }

    #[inline]
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
        TranslationBase {
            vector: Interpolate::linear_interpolate(t, &x1.vector, &x2.vector)
        }
    }
}

impl<N, D: DimName, S> Interpolate for RotationBase<N, D, S> where N: Scalar,
                                                                   S: Storage<N, D, D>,
                                                                   Matrix<N, D, D, S>: Interpolate {
    #[inline]
    fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
        RotationBase::from_matrix_unchecked(Interpolate::barycentric_interpolate(
            u, ux.matrix(),
            v, vx.matrix(),
            w, wx.matrix(),
        ))
    }

    #[inline]
    fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
        RotationBase::from_matrix_unchecked(Interpolate::linear_interpolate(t, x1.matrix(), x2.matrix()))
    }
}

// Format of this was taken from nalgebra/core/construction.rs
macro_rules! nalgebra_matrix_uniforms {
    ($($R: ty, $C: ty, $($args: ident:($irow: expr,$icol: expr)),*);* $(;)*) => {$(
        impl<N, S> $crate::Interpolate for Matrix<N, $R, $C, S>
            where N: Scalar + $crate::Interpolate,
                  S: OwnedStorage<N, $R, $C>,
                  S::Alloc: OwnedAllocator<N, $R, $C, S> {
            #[inline]
            fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
                unsafe {
                    let mut res = Self::new_uninitialized();

                    $(
                        *res.get_unchecked_mut($irow, $icol) = $crate::Interpolate::barycentric_interpolate(
                            u, ux.get_unchecked($irow, $icol),
                            v, vx.get_unchecked($irow, $icol),
                            w, wx.get_unchecked($irow, $icol)
                        );
                    )*

                    res
                }
            }

            #[inline]
            fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
                unsafe {
                    let mut res = Self::new_uninitialized();

                    $(
                        *res.get_unchecked_mut($irow, $icol) = $crate::Interpolate::linear_interpolate(
                            t, x1.get_unchecked($irow, $icol), x2.get_unchecked($irow, $icol)
                        );
                    )*

                    res
                }
            }
        }
    )*}
}

nalgebra_matrix_uniforms!(
    /*
     * Square matrices 1 .. 6.
     */
    U2, U2, m11:(0,0), m12:(0,1),
            m21:(1,0), m22:(1,1);
    U3, U3, m11:(0,0), m12:(0,1), m13:(0,2),
            m21:(1,0), m22:(1,1), m23:(1,2),
            m31:(2,0), m32:(2,1), m33:(2,2);
    U4, U4, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3);
    U5, U5, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3), m45:(3,4),
            m51:(4,0), m52:(4,1), m53:(4,2), m54:(4,3), m55:(4,4);
    U6, U6, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4), m16:(0,5),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4), m26:(1,5),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4), m36:(2,5),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3), m45:(3,4), m46:(3,5),
            m51:(4,0), m52:(4,1), m53:(4,2), m54:(4,3), m55:(4,4), m56:(4,5),
            m61:(5,0), m62:(5,1), m63:(5,2), m64:(5,3), m65:(5,4), m66:(5,5);

    /*
     * Rectangular matrices with 2 rows.
     */
    U2, U3, m11:(0,0), m12:(0,1), m13:(0,2),
            m21:(1,0), m22:(1,1), m23:(1,2);
    U2, U4, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3);
    U2, U5, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4);
    U2, U6, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4), m16:(0,5),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4), m26:(1,5);

    /*
     * Rectangular matrices with 3 rows.
     */
    U3, U2, m11:(0,0), m12:(0,1),
            m21:(1,0), m22:(1,1),
            m31:(2,0), m32:(2,1);
    U3, U4, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3);
    U3, U5, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4);
    U3, U6, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4), m16:(0,5),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4), m26:(1,5),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4), m36:(2,5);

    /*
     * Rectangular matrices with 4 rows.
     */
    U4, U2, m11:(0,0), m12:(0,1),
            m21:(1,0), m22:(1,1),
            m31:(2,0), m32:(2,1),
            m41:(3,0), m42:(3,1);
    U4, U3, m11:(0,0), m12:(0,1), m13:(0,2),
            m21:(1,0), m22:(1,1), m23:(1,2),
            m31:(2,0), m32:(2,1), m33:(2,2),
            m41:(3,0), m42:(3,1), m43:(3,2);
    U4, U5, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3), m45:(3,4);
    U4, U6, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4), m16:(0,5),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4), m26:(1,5),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4), m36:(2,5),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3), m45:(3,4), m46:(3,5);

    /*
     * Rectangular matrices with 5 rows.
     */
    U5, U2, m11:(0,0), m12:(0,1),
            m21:(1,0), m22:(1,1),
            m31:(2,0), m32:(2,1),
            m41:(3,0), m42:(3,1),
            m51:(4,0), m52:(4,1);
    U5, U3, m11:(0,0), m12:(0,1), m13:(0,2),
            m21:(1,0), m22:(1,1), m23:(1,2),
            m31:(2,0), m32:(2,1), m33:(2,2),
            m41:(3,0), m42:(3,1), m43:(3,2),
            m51:(4,0), m52:(4,1), m53:(4,2);
    U5, U4, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3),
            m51:(4,0), m52:(4,1), m53:(4,2), m54:(4,3);
    U5, U6, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4), m16:(0,5),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4), m26:(1,5),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4), m36:(2,5),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3), m45:(3,4), m46:(3,5),
            m51:(4,0), m52:(4,1), m53:(4,2), m54:(4,3), m55:(4,4), m56:(4,5);

    /*
     * Rectangular matrices with 6 rows.
     */
    U6, U2, m11:(0,0), m12:(0,1),
            m21:(1,0), m22:(1,1),
            m31:(2,0), m32:(2,1),
            m41:(3,0), m42:(3,1),
            m51:(4,0), m52:(4,1),
            m61:(5,0), m62:(5,1);
    U6, U3, m11:(0,0), m12:(0,1), m13:(0,2),
            m21:(1,0), m22:(1,1), m23:(1,2),
            m31:(2,0), m32:(2,1), m33:(2,2),
            m41:(3,0), m42:(3,1), m43:(3,2),
            m51:(4,0), m52:(4,1), m53:(4,2),
            m61:(5,0), m62:(5,1), m63:(5,2);
    U6, U4, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3),
            m51:(4,0), m52:(4,1), m53:(4,2), m54:(4,3),
            m61:(5,0), m62:(5,1), m63:(5,2), m64:(5,3);
    U6, U5, m11:(0,0), m12:(0,1), m13:(0,2), m14:(0,3), m15:(0,4),
            m21:(1,0), m22:(1,1), m23:(1,2), m24:(1,3), m25:(1,4),
            m31:(2,0), m32:(2,1), m33:(2,2), m34:(2,3), m35:(2,4),
            m41:(3,0), m42:(3,1), m43:(3,2), m44:(3,3), m45:(3,4),
            m51:(4,0), m52:(4,1), m53:(4,2), m54:(4,3), m55:(4,4),
            m61:(5,0), m62:(5,1), m63:(5,2), m64:(5,3), m65:(5,4);

    /*
     * Row vectors 1 .. 6.
     */
    U1, U1, x:(0,0);
    U1, U2, x:(0,0), y:(0,1);
    U1, U3, x:(0,0), y:(0,1), z:(0,2);
    U1, U4, x:(0,0), y:(0,1), z:(0,2), w:(0,3);
    U1, U5, x:(0,0), y:(0,1), z:(0,2), w:(0,3), a:(0,4);
    U1, U6, x:(0,0), y:(0,1), z:(0,2), w:(0,3), a:(0,4), b:(0,5);

    /*
     * Column vectors 1 .. 6.
     */
    U2, U1, x:(0,0), y:(1,0);
    U3, U1, x:(0,0), y:(1,0), z:(2,0);
    U4, U1, x:(0,0), y:(1,0), z:(2,0), w:(3,0);
    U5, U1, x:(0,0), y:(1,0), z:(2,0), w:(3,0), a:(4,0);
    U6, U1, x:(0,0), y:(1,0), z:(2,0), w:(3,0), a:(4,0), b:(5,0);
);