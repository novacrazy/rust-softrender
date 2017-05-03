//! Useful compatibility with the `image` crate

use image;

use ::pixel::RGBAf32Pixel;
use ::render::FrameBuffer;

/// Additional functionality for copying a framebuffer into an image
pub trait ImageFrameBuffer<P, C> where P: image::Pixel {
    /// Copies the floating point color component of the framebuffer into a `Rgba<u8>` image.
    ///
    /// This clamps all color channels between 0.0 and 1.0,
    /// so tonemapping of HDR colors should be done before this to avoid any undesired behavior.
    fn copy_to_image(&self) -> Option<image::ImageBuffer<P, C>>;
}

impl ImageFrameBuffer<image::Rgba<u8>, Vec<u8>> for FrameBuffer<RGBAf32Pixel> {
    fn copy_to_image(&self) -> Option<image::RgbaImage> {
        let color_buffer = self.color_buffer();

        let mut res = Vec::with_capacity(color_buffer.len() * 4);

        for color in color_buffer {
            res.push((color.r.max(0.0).min(1.0) * 255.0).floor() as u8);
            res.push((color.g.max(0.0).min(1.0) * 255.0).floor() as u8);
            res.push((color.b.max(0.0).min(1.0) * 255.0).floor() as u8);
            res.push((color.a.max(0.0).min(1.0) * 255.0).floor() as u8);
        }

        image::RgbaImage::from_raw(self.width(), self.height(), res)
    }
}