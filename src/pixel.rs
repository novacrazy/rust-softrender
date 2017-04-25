//! Pixel definition and operations

pub trait Pixel: Send + Sync {}

/// The most dead simple f32 pixel representation
#[derive(Debug, Clone, Copy)]
pub struct F32RGBAPixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Pixel for F32RGBAPixel {}