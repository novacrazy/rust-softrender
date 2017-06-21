pub mod attachments;

pub use self::attachments::Attachments;

use self::attachments::{Depth, Stencil};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    #[inline]
    pub fn new(width: u32, height: u32) -> Dimensions {
        Dimensions { width, height }
    }

    #[inline]
    pub fn pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[inline]
    pub fn valid(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }
}

pub trait Framebuffer: 'static {
    type Attachments: Attachments;
    fn dimensions(&self) -> Dimensions;
    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color);
}

pub struct RenderBuffer<A: Attachments> {
    dimensions: Dimensions,
    stencil: <A::Stencil as Stencil>::Config,
    /// Interlaced framebuffer for more cache-friendly access
    pub ( crate ) buffer: Vec<(A::Color, A::Depth, <A::Stencil as Stencil>::Type)>,
}

impl<A> Framebuffer for RenderBuffer<A> where A: Attachments {
    type Attachments = A;

    fn dimensions(&self) -> Dimensions { self.dimensions }

    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color) {
        for mut a in &mut self.buffer {
            *a = (color, <A::Depth as Depth>::far(), Default::default());
        }
    }
}

/// Declares a new texture buffer type with the specified color buffer attachments, as if they were a struct.
///
/// Unlike with the simple `RenderBuffer`, texture buffers store each color component in a separate buffer. This allows
/// for the rendered images to be re-used as textures without copying, which can result in a net performance gain.
///
/// However, because of the extra complexity, a macro is used to simplify the creation of these.
///
/// See the `predefined` module for an example of a texture buffer with a single color attachment, which was created with the macro invocation:
///
/// ```ignore
/// declare_texture_buffer! {
///     /// Texture Buffer with a single RGBA 32-bit Floating Point color
///     pub struct RGBAf32TextureBuffer {
///         /// Primary color buffer
///         pub color: RGBAf32Color,
///     }
/// }
/// ```
///
/// Because the inner buffers are an implementation detail,
/// the attributes given to the fields are placed on their accessor functions.
#[macro_export]
macro_rules! declare_texture_buffer {
    (
        $(#[$($struct_attrs:tt)*])*
        pub struct $buffer_name:ident {
            $(
                $(#[$($field_attrs:tt)*])*
                pub $color_name:ident: $color_ty:ty,
            )+
        }
    ) => {
        $(#[$($struct_attrs)*])*
        pub struct $buffer_name<A: $crate::attachments::Attachments>
            where <A as $crate::attachments::Attachments>::Color: $crate::attachments::EmptyAttachment {
            $($color_name: Vec<$color_ty>,)+
            dimensions: $crate::framebuffer::Dimensions,
            stencil: <A::Stencil as $crate::attachments::Stencil>::Config,
            buffer: Vec<(A::Depth, <A::Stencil as $crate::attachments::Stencil>::Type)>,
        }

        impl<A: $crate::attachments::Attachments> $buffer_name<A>
            where <A as $crate::attachments::Attachments>::Color: $crate::attachments::EmptyAttachment {
            pub fn new(dimensions: $crate::framebuffer::Dimensions) -> $buffer_name<A> {
                let len = dimensions.width  as usize *
                          dimensions.height as usize;

                $buffer_name {
                    $($color_name: vec![<$color_ty as $crate::attachments::Color>::empty(); len],)+
                    dimensions,
                    stencil: Default::default(),
                    buffer: vec![Default::default(); len]
                }
            }

            $(
                $(#[$($field_attrs)*])*
                #[inline]
                pub fn $color_name(&self) -> &[$color_ty] { &self.$color_name }
            )+
        }

        impl<A: $crate::attachments::Attachments> $crate::framebuffer::Framebuffer for $buffer_name<A>
            where <A as $crate::attachments::Attachments>::Color: $crate::attachments::EmptyAttachment {
            type Attachments = $crate::attachments::ColorDepthStencilAttachments<($($color_ty,)+), A::Depth, A::Stencil>;

            fn dimensions(&self) -> $crate::framebuffer::Dimensions { self.dimensions }

            fn clear(&mut self, color: <Self::Attachments as $crate::attachments::Attachments>::Color) {
                // Destructure the tuple into its individual attachments, by name, so they can be used one by one.
                let ($($color_name,)+) = color;

                $(
                    // Go through each buffer and overwrite everything with the destructured colors, by name.
                    for mut c in &mut self.$color_name {
                        *c = $color_name;
                    }
                )+

                for mut a in &mut self.buffer {
                    *a = (<A::Depth as $crate::attachments::Depth>::far(), Default::default());
                }
            }
        }
    }
}

pub mod predefined {
    use attachments::color::predefined::formats::RGBAf32Color;

    declare_texture_buffer! {
        /// Texture Buffer with a single RGBA 32-bit Floating Point color
        pub struct RGBAf32TextureBuffer {
            /// Primary color buffer
            pub color: RGBAf32Color,
        }
    }
}