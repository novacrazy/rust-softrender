use ::pixel::Pixel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    Nearest,
    Bilinear,
}

#[derive(Debug, Clone, Copy)]
pub enum Edge<P: Pixel> {
    /// Equivalent to `GL_CLAMP_TO_EDGE`
    Clamp,
    /// Equivalent to `GL_REPEAT`
    Wrap,
    /// Equivalent to `GL_CLAMP_TO_BORDER`
    Border(P)
}

pub trait Texture<P: Pixel> {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn pixel(&self, x: u32, y: u32) -> &P;

    fn sample(&self, x: f32, y: f32, filter: Filter) -> P {
        unimplemented!()
    }
}