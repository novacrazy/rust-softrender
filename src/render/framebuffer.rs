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
    viewport: (f32, f32),
    cache: Vec<FrameBuffer<P>>,
}

/// Default depth value, equal to the farthest away anything can be.
///
/// Note that due to how floating point numbers work,
/// depth values become less precise the farther away the object is.
pub const DEFAULT_DEPTH_VALUE: f32 = ::std::f32::MAX;

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
            viewport: (width as f32, height as f32),
            cache: Vec::new(),
        }
    }

    /// Create a clone of the framebuffer with all the same properties but with pixels
    /// being uninitialized and with the `DEFAULT_DEPTH_VALUE` in the depth buffer
    pub fn empty_clone(&mut self) -> FrameBuffer<P> {
        if let Some(fb) = self.cache.pop() { fb } else {
            FrameBuffer {
                width: self.width,
                height: self.height,
                depth: self.depth.clone(),
                color: {
                    let mut empty = Vec::with_capacity(self.color.len());

                    unsafe { empty.set_len(self.color.len()); }

                    empty
                },
                viewport: self.viewport,
                cache: Vec::new(),
            }
        }
    }

    pub fn cache_empty_clone(&mut self, fb: FrameBuffer<P>) {
        self.cache.push(fb)
    }

    /// Merges `self` into another framebuffer, taking into account the depth buffer and pixel blending.
    pub fn merge_into(&mut self, mut other: &mut FrameBuffer<P>, blend_func: &Box<Fn(P, P) -> P + Send + Sync>) {
        let (pcolor, pdepth) = self.buffers_mut();
        let (fcolor, fdepth) = other.buffers_mut();

        debug_assert_eq!(pcolor.len(), fcolor.len());
        debug_assert_eq!(pdepth.len(), fdepth.len());
        debug_assert_eq!(pcolor.len(), fdepth.len());

        for i in 0..pcolor.len() {
            unsafe {
                let fd = fdepth.get_unchecked_mut(i);
                let pd = pdepth.get_unchecked_mut(i);

                if *fd > *pd {
                    *fd = *pd;

                    let pc = pcolor.get_unchecked(i);

                    let fc = fcolor.get_unchecked_mut(i);
                    *fc = (*blend_func)(*pc, *fc);
                } else {
                    // synchronize the depth values
                    *pd = *fd;
                }
            }
        }
    }

    /// Sets all depth and color values to their default
    pub fn clear(&mut self) where P: Default {
        self.clear_with(P::default())
    }

    /// Sets all depth values to their default and the color values to the given pixel.
    pub fn clear_with(&mut self, pixel: P) {
        self.clear_depth();

        for mut pv in &mut self.color { *pv = pixel; }

        for pf in &mut self.cache {
            pf.clear_depth();
        }
    }

    pub fn clear_depth(&mut self) {
        for mut dv in &mut self.depth { *dv = DEFAULT_DEPTH_VALUE; }
    }

    /// Get a reference to the color buffer
    pub fn color_buffer(&self) -> &[P] { &self.color }
    /// Get a mutable reference to the color buffer
    pub fn color_buffer_mut(&mut self) -> &mut [P] { &mut self.color }

    /// Get a reference to the depth buffer
    pub fn depth_buffer(&self) -> &[f32] { &self.depth }
    /// Get a mutable reference to the depth buffer
    pub fn depth_buffer_mut(&mut self) -> &mut [f32] { &mut self.depth }

    /// Returns references to all buffers at once
    #[inline]
    pub fn buffers(&self) -> (&[P], &[f32]) {
        (&self.color, &self.depth)
    }

    /// Returns mutable references to all buffers at once
    #[inline]
    pub fn buffers_mut(&mut self) -> (&mut [P], &mut [f32]) {
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

    /// Get a mutable reference to both the depth and color values at the given coordinate.
    ///
    /// No bounds checking is performed for performance reasons,
    /// so bounds should be checked elsewhere.
    #[inline]
    pub unsafe fn pixel_depth_mut(&mut self, x: u32, y: u32) -> (&mut P, &mut f32) {
        let i = x as usize + y as usize * self.width as usize;
        (self.color.get_unchecked_mut(i), self.depth.get_unchecked_mut(i))
    }
}