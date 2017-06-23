use ::error::{RenderError, RenderResult};

use super::Coordinate;

/// Defines types with set dimensions
pub trait HasDimensions {
    /// Returns the dimensions of the object
    fn dimensions(&self) -> Dimensions;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    #[inline]
    pub fn new(width: u32, height: u32) -> Dimensions {
        Dimensions { width, height }
    }

    #[inline]
    pub fn pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[inline]
    pub fn valid(&self, coord: Coordinate) -> bool {
        let Coordinate { x, y } = coord;

        x < self.width && y < self.height
    }

    #[inline]
    pub fn check_valid(&self, coord: Coordinate) -> RenderResult<()> {
        if self.valid(coord) { Ok(()) } else {
            throw!(RenderError::InvalidPixelCoordinate)
        }
    }
}
