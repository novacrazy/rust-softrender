extern crate image;

pub mod color;
pub mod pixelbuffer;

#[cfg(test)]
mod test {
    use super::*;
    use super::image::{Rgba, RgbaImage};

    use ::texture::TextureRead;
    use ::attachments::predefined::EmptyAttachments;

    fn assert_texture<T: TextureRead>(_: T) {}

    #[test]
    fn test_image_texture() {
        let t = RgbaImage::from_raw(1, 1, vec![Default::default(); ::std::mem::size_of::<Rgba<u8>>()]).unwrap();

        assert_texture(t)
    }
}