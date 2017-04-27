use image;

use ::pixel::RGBAf32Pixel;
use ::render::FrameBuffer;

pub trait ImageFrameBuffer<P, C> where P: image::Pixel {
    fn copy_to_image(&self) -> Option<image::ImageBuffer<P, C>>;
}

impl ImageFrameBuffer<image::Rgba<u8>, Vec<u8>> for FrameBuffer<RGBAf32Pixel> {
    fn copy_to_image(&self) -> Option<image::RgbaImage> {
        let color_buffer: &Vec<RGBAf32Pixel> = self.color_buffer();

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