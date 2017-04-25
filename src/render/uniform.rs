//! Types and traits for uniform variables

use nalgebra::*;

use std::ops::{Add, Mul};

/// Describes a type that can be interpolated with barycentric coordinates.
///
/// This is required for any rasterization to occur.
///
/// See [This document](https://classes.soe.ucsc.edu/cmps160/Fall10/resources/barycentricInterpolation.pdf) for more information
pub trait BarycentricInterpolation {
    fn interpolate(a: f32, a1: f32, x1: Self, a2: f32, x2: Self, a3: f32, x3: Self) -> Self;
}

impl<T> BarycentricInterpolation for T where T: Add<Output=Self> + Add<f32, Output=Self> + Mul<f32, Output=Self> {
    fn interpolate(a: f32, a1: f32, x1: Self, a2: f32, x2: Self, a3: f32, x3: Self) -> Self {
        // In debug mode, assert that the total combined area is approximately equal to the actual area
        debug_assert!(((a1 + a2 + a3) - a).abs() <= 0.1);

        (x1 * a1 + x2 * a2 + x3 * a3) * (1.0 / a)
    }
}