//! Minimalist framebuffer structure with an emphasis on performance

use ::pixel::Pixel;

/// Minimalist framebuffer structure with an emphasis on performance
///
/// It contains a color buffer and a depth buffer
pub struct FrameBuffer<P: Pixel> {
    width: u32,
    height: u32,
    depth: Vec<f32>,
    color: Vec<P>,
    blend_func: Box<Fn(P, P) -> P + Send + Sync>
}

impl<P: Pixel> FrameBuffer<P> {
    /// Get the width of the framebuffer in pixels
    #[inline(always)]
    pub fn width(&self) -> u32 { self.width }

    /// Get the height of the framebuffer in pixels
    #[inline(always)]
    pub fn height(&self) -> u32 { self.height }

    /// Blend two pixels together with the set blend functions
    #[inline(always)]
    pub fn blend(&self, source: P, destination: P) -> P {
        (*self.blend_func)(source, destination)
    }

    /// Set the pixel blend function
    pub fn set_blend_function<F>(&mut self, f: F) where F: Fn(P, P) -> P + Send + Sync + 'static {
        self.blend_func = Box::new(f)
    }

    /// Check if some x and y coordinate is a valid pixel coordinate
    #[inline(always)]
    pub fn check_coordinate(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    /// Get a reference to the pixel at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn pixel(&self, x: u32, y: u32) -> &P {
        self.color.get_unchecked((x + y * self.width) as usize)
    }

    /// Get a mutable reference to the pixel at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn pixel_mut(&mut self, x: u32, y: u32) -> &mut P {
        self.color.get_unchecked_mut((x + y * self.width) as usize)
    }

    /// Get a reference to the depth value at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn depth(&self, x: u32, y: u32) -> &f32 {
        self.depth.get_unchecked((x + y * self.width) as usize)
    }

    /// Get a mutable reference to the depth value at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn depth_mut(&mut self, x: u32, y: u32) -> &mut f32 {
        self.depth.get_unchecked_mut((x + y * self.width) as usize)
    }
}