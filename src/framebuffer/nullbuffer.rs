use super::{Framebuffer, Dimensions, attachments};

#[derive(Debug, Clone, Copy)]
pub struct NullFramebuffer {
    dimensions: Dimensions,
}

impl Framebuffer for NullFramebuffer {
    type Attachments = attachments::predefined::EmptyAttachments;

    fn dimensions(&self) -> Dimensions { self.dimensions }

    fn clear(&mut self, color: ()) {}

    unsafe fn get_pixel_unchecked(&self, _: usize) -> () { () }
    unsafe fn set_pixel_unchecked(&mut self, _: usize, _: ()) {}
}

impl NullFramebuffer {
    pub fn new() -> NullFramebuffer {
        NullFramebuffer {
            dimensions: Dimensions::new(0, 0)
        }
    }

    pub fn with_dimensions(dimensions: Dimensions) -> NullFramebuffer {
        NullFramebuffer { dimensions }
    }
}