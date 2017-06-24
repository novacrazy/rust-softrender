#[macro_use]
extern crate softrender;

use softrender::attachments::color::predefined::formats::RGBAf32Color;

declare_texture_buffer! {
    /// Texture Buffer with a single RGBA 32-bit Floating Point color.
    ///
    /// Unlike the `RenderBuffer`, texture buffers can easily have one or more color attachment
    /// be reused as a texture for a subsequent render.
    pub struct RGBAf32TextureBuffer {
        /// Primary color buffer
        pub color: RGBAf32Color,
    }
}