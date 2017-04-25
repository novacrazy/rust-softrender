use super::pixel::Pixel;

#[derive(Clone)]
pub struct FrameBuffer<P: Pixel> {
    width: u32,
    height: u32,
    depth: Vec<f32>,
    color: Vec<P>,
}

impl<P: Pixel> FrameBuffer<P> {
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    #[inline(always)]
    pub fn check_coordinate(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    #[inline]
    pub unsafe fn pixel(&self, x: u32, y: u32) -> &P {
        self.color.get_unchecked((x + y * self.width) as usize)
    }

    #[inline]
    pub unsafe fn pixel_mut(&mut self, x: u32, y: u32) -> &mut P {
        self.color.get_unchecked_mut((x + y * self.width) as usize)
    }

    #[inline]
    pub unsafe fn depth(&self, x: u32, y: u32) -> &f32 {
        self.depth.get_unchecked((x + y * self.width) as usize)
    }

    #[inline]
    pub unsafe fn depth_mut(&mut self, x: u32, y: u32) -> &mut f32 {
        self.depth.get_unchecked_mut((x + y * self.width) as usize)
    }
}