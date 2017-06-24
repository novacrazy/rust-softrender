use ::geometry::{Dimensions, HasDimensions};
use ::pixels::{PixelBuffer, PixelRead, PixelWrite};

use super::{Framebuffer, attachments, Attachments};

#[derive(Debug, Clone, Copy)]
pub struct NullFramebuffer {
    dimensions: Dimensions,
}

impl HasDimensions for NullFramebuffer {
    fn dimensions(&self) -> Dimensions { self.dimensions }
}

impl PixelBuffer for NullFramebuffer {
    type Color = <<Self as Framebuffer>::Attachments as Attachments>::Color;
}

impl PixelRead for NullFramebuffer {
    unsafe fn get_pixel_unchecked(&self, _: usize) -> () { () }
}

impl PixelWrite for NullFramebuffer {
    unsafe fn set_pixel_unchecked(&mut self, _: usize, _: ()) {}
}

impl Framebuffer for NullFramebuffer {
    type Attachments = attachments::predefined::EmptyAttachments;

    fn clear(&mut self, _: ()) {}
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