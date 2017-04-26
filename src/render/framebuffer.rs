//! Minimalist framebuffer structure with an emphasis on performance
use std::sync::Arc;

use ::pixel::Pixel;

/// Minimalist framebuffer structure with an emphasis on performance
///
/// It contains a color buffer and a depth buffer
pub struct FrameBuffer<P: Pixel> {
    width: u32,
    height: u32,
    depth: Vec<f32>,
    color: Vec<P>,
    blend_func: Arc<Box<Fn(P, P) -> P + Send + Sync>>,
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
            blend_func: Arc::new(Box::new(|s, _| s)),
            viewport: (width as f32, height as f32)
        }
    }

    pub fn empty_clone(&self) -> FrameBuffer<P> {
        let len = self.width as usize * self.height as usize;

        FrameBuffer {
            width: self.width,
            height: self.height,
            depth: vec![DEFAULT_DEPTH_VALUE; len],
            color: vec![P::empty(); len],
            blend_func: self.blend_func.clone(),
            viewport: self.viewport
        }
    }

    /// Sets all depth and color values to their default
    pub fn clear(&mut self) where P: Default {
        self.clear_with(P::default())
    }

    /// Sets all depth values to their default and the color values to the given pixel.
    pub fn clear_with(&mut self, pixel: P) {
        for mut dv in &mut self.depth { *dv = DEFAULT_DEPTH_VALUE; }
        for mut pv in &mut self.color { *pv = pixel; }
    }

    /// Get a reference to the color buffer
    pub fn color_buffer(&self) -> &Vec<P> { &self.color }
    /// Get a mutable reference to the color buffer
    pub fn color_buffer_mut(&mut self) -> &mut Vec<P> { &mut self.color }

    /// Get a reference to the depth buffer
    pub fn depth_buffer(&self) -> &Vec<f32> { &self.depth }
    /// Get a mutable reference to the depth buffer
    pub fn depth_buffer_mut(&mut self) -> &mut Vec<f32> { &mut self.depth }

    /// Returns references to all buffers at once
    pub fn buffers(&self) -> (&Vec<P>, &Vec<f32>) {
        (&self.color, &self.depth)
    }

    /// Returns mutable references to all buffers at once
    pub fn buffers_mut(&mut self) -> (&mut Vec<P>, &mut Vec<f32>) {
        (&mut self.color, &mut self.depth)
    }

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

    pub fn blend_func(&self) -> Arc<Box<Fn(P, P) -> P + Send + Sync>> {
        self.blend_func.clone()
    }

    /// Set the pixel blend function
    pub fn set_blend_function<F>(&mut self, f: F) where F: Fn(P, P) -> P + Send + Sync + 'static {
        self.blend_func = Arc::new(Box::new(f))
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