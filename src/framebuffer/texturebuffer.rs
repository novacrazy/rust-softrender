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
        #[derive(Clone)]
        pub struct $buffer_name<A: $crate::attachments::Attachments>
            where <A as $crate::attachments::Attachments>::Color: $crate::attachments::EmptyAttachment {
            $($color_name: Vec<$color_ty>,)+
            dimensions: $crate::framebuffer::Dimensions,
            stencil: <A::Stencil as $crate::attachments::Stencil>::Config,
            buffer: Vec<(A::Depth, <A::Stencil as $crate::attachments::Stencil>::Type)>,
        }

        impl<A: $crate::attachments::Attachments> $buffer_name<A>
            where <A as $crate::attachments::Attachments>::Color: $crate::attachments::EmptyAttachment {
            pub fn new() -> $buffer_name<A> {
                $buffer_name {
                    $($color_name: Vec::new(),)+
                    dimensions: $crate::framebuffer::Dimensions::new(0, 0),
                    stencil: Default::default(),
                    buffer: Vec::new(),
                }
            }

            pub fn with_dimensions(dimensions: $crate::framebuffer::Dimensions) -> $buffer_name<A> {
                let pixels = dimensions.pixels();

                $buffer_name {
                    $($color_name: vec![<$color_ty as $crate::attachments::Color>::empty(); pixels],)+
                    dimensions,
                    stencil: Default::default(),
                    buffer: vec![(<<A as $crate::attachments::Attachments>::Depth as $crate::attachments::Depth>::far(),
                                  Default::default()); pixels],
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


            unsafe fn get_pixel_unchecked(&self, index: usize) -> <Self::Attachments as $crate::attachments::Attachments>::Color {
                ($(*self.$color_name.get_unchecked(index),)+)
            }

            unsafe fn set_pixel_unchecked(&mut self, index: usize, color: <Self::Attachments as $crate::attachments::Attachments>::Color) {
                let ($($color_name,)+) = color;

                $(*self.$color_name.get_unchecked_mut(index) = $color_name;)+
            }
        }
    }
}

pub mod predefined {
    use ::attachments::color::predefined::formats::RGBAf32Color;

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
}