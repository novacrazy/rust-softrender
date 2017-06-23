

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
    pub fn into_index(self) -> usize {
        let Coordinate { x, y } = self;
        x as usize * y as usize
    }
}