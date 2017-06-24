use super::Dimensions;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

impl Coordinate {
    #[inline]
    pub fn new(x: u32, y: u32) -> Coordinate {
        Coordinate { x, y }
    }

    #[inline]
    pub fn into_index(self, dimensions: Dimensions) -> usize {
        let Coordinate { x, y } = self;
        x as usize + y as usize * dimensions.height as usize
    }

    #[inline]
    pub fn from_index(index: usize, dimensions: Dimensions) -> Coordinate {
        let Dimensions { width, height } = dimensions;

        let x = index % width as usize;
        let y = (index - x) / height as usize;

        Coordinate { x: x as u32, y: y as u32 }
    }
}