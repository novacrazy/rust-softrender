use ::error::{RenderError, RenderResult};

use super::Coordinate;

/// Defines types with set dimensions
pub trait HasDimensions {
    /// Returns the dimensions of the object
    fn dimensions(&self) -> Dimensions;

    /// Checks if the given coordinate is within the dimension bounds of the current object
    #[inline]
    fn in_bounds(&self, coord: Coordinate) -> bool {
        self.dimensions().in_bounds(coord)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    #[inline(always)]
    pub fn new(width: u32, height: u32) -> Dimensions {
        Dimensions { width, height }
    }

    /// Returns the number of pixels as `usize` by multiplying the current width and height
    #[inline]
    pub fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Checks if the given coordinate is within the dimension bounds
    #[inline]
    pub fn in_bounds(&self, coord: Coordinate) -> bool {
        coord.x < self.width && coord.y < self.height
    }
}
