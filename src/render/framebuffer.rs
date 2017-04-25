//! Minimalist framebuffer structure with an emphasis on performance
use rayon;

use ::pixel::Pixel;

/// Minimalist framebuffer structure with an emphasis on performance
///
/// It contains a color buffer and a depth buffer
pub struct FrameBuffer<P: Pixel> {
    width: u32,
    height: u32,
    depth: Vec<f32>,
    color: Vec<P>,
    blend_func: Box<Fn(P, P) -> P + Send + Sync>,
    viewport: (f32, f32),
}

pub const DEFAULT_DEPTH_VALUE: f32 = 1000000.0;

impl<P: Pixel> FrameBuffer<P> {
    /// Create a new framebuffer with the default pixel.
    pub fn new(width: u32, height: u32) -> FrameBuffer<P> where P: Default {
        FrameBuffer::new_with(width, height, P::default())
    }

    /// Creates a new framebuffer with the given pixel.
    pub fn new_with(width: u32, height: u32, pixel: P) -> FrameBuffer<P> {
        let len = width as usize * height as usize;
        FrameBuffer {
            width: width,
            height: height,
            depth: vec![DEFAULT_DEPTH_VALUE; len],
            color: vec![pixel; len],
            blend_func: Box::new(|s, _| s),
            viewport: (width as f32, height as f32)
        }
    }

    /// Sets all depth and color values to their default
    pub fn clear(&mut self) where P: Default {
        self.clear_with(P::default())
    }

    /// Sets all depth values to their default and the color values to the given pixel.
    pub fn clear_with(&mut self, pixel: P) {
        let ref mut depth = self.depth;
        let ref mut color = self.color;

        // Might as well clear them both in parallel.
        // Although I'll have to test if it's faster this way or to just do them sequentially,
        // because of thread scheduling overhead and so forth.
        rayon::join(|| { for mut dv in depth { *dv = DEFAULT_DEPTH_VALUE; } },
                    || { for mut pv in color { *pv = pixel; } });
    }

    /// Get a reference to the color buffer
    pub fn color_buffer(&self) -> &Vec<P> { &self.color }

    // Get a reference to the depth buffer
    pub fn depth_buffer(&self) -> &Vec<f32> { &self.depth }

    /// Get the projection viewport dimensions
    #[inline(always)]
    pub fn viewport(&self) -> (f32, f32) { self.viewport }

    /// Set the projection viewport dimensions
    pub fn set_viewport(&mut self, viewport: (f32, f32)) {
        self.viewport = viewport;
    }

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
        self.color.get_unchecked(x as usize + y as usize * self.width as usize)
    }

    /// Get a mutable reference to the pixel at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn pixel_mut(&mut self, x: u32, y: u32) -> &mut P {
        self.color.get_unchecked_mut(x as usize + y as usize * self.width as usize)
    }

    /// Get a reference to the depth value at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn depth(&self, x: u32, y: u32) -> &f32 {
        self.depth.get_unchecked(x as usize + y as usize * self.width as usize)
    }

    /// Get a mutable reference to the depth value at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn depth_mut(&mut self, x: u32, y: u32) -> &mut f32 {
        self.depth.get_unchecked_mut(x as usize + y as usize * self.width as usize)
    }
}