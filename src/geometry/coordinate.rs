use std::ops::{Add, AddAssign};

use nalgebra::Vector2;
use nalgebra::coordinates::XY;

use super::Dimensions;

/// Simple 2D Coordinate structure. Easily converts to/from nalgebra's `Vector2D<u32>` for more complex operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Coordinate {
    /// x-coordinate
    pub x: u32,
    /// y-coordinate
    pub y: u32,
}

impl Coordinate {
    /// Create new `Coordinate` from `x` and `y` components
    #[inline]
    pub fn new(x: u32, y: u32) -> Coordinate {
        Coordinate { x, y }
    }

    /// Construct a `Coordinate` from a 2D Vector
    #[inline]
    pub fn from_vector(v: Vector2<u32>) -> Coordinate {
        let XY { x, y } = *v;
        Coordinate::new(x, y)
    }

    /// Convert the `Coordinate` to a 2D Veector.
    #[inline]
    pub fn into_vector(self) -> Vector2<u32> {
        let Coordinate { x, y } = self;

        Vector2::new(x, y)
    }

    /// Convert a 2D coordinate into a 1D array index using the given `Dimensions`
    #[inline]
    pub fn into_index(self, dimensions: Dimensions) -> usize {
        let Coordinate { x, y } = self;
        x as usize + y as usize * dimensions.height as usize
    }

    /// Convert a 1D array index into a 2D coordinate using the given `Dimensions`
    #[inline]
    pub fn from_index(index: usize, dimensions: Dimensions) -> Coordinate {
        let Dimensions { width, height } = dimensions;

        let x = index % width as usize;
        let y = (index - x) / height as usize;

        Coordinate { x: x as u32, y: y as u32 }
    }
}

impl From<Vector2<u32>> for Coordinate {
    #[inline(always)]
    fn from(v: Vector2<u32>) -> Coordinate {
        Coordinate::from_vector(v)
    }
}

impl From<Coordinate> for Vector2<u32> {
    #[inline(always)]
    fn from(coord: Coordinate) -> Vector2<u32> {
        coord.into_vector()
    }
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Coordinate) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}