use ::error::{RenderError, RenderResult};

pub mod attachments;

pub use self::attachments::Attachments;

use self::attachments::{Depth, Stencil};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
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
    pub fn valid(&self, coord: Coordinate) -> bool {
        let Coordinate { x, y } = coord;

        x < self.width && y < self.height
    }

    #[inline]
    pub fn check_valid(&self, coord: Coordinate) -> RenderResult<()> {
        if self.valid(coord) { Ok(()) } else {
            throw!(RenderError::InvalidPixelCoordinate)
        }
    }
}

impl Coordinate {
    #[inline]
    pub fn new(x: u32, y: u32) -> Coordinate {
        Coordinate { x, y }
    }

    #[inline]
    pub fn into_index(self) -> usize {
        let Coordinate { x, y } = self;
        x as usize * y as usize
    }
}

/// Immutable reference to a pixel.
///
/// Provides a read-only accessor for the pixel at the coordinates given at creation.
pub struct PixelRef<'a, F>(usize, &'a F) where F: Framebuffer;

/// Mutable reference to a pixel
///
/// Provides a writable accessor for the pixel at the coordinates given at creation.
pub struct PixelMut<'a, F>(usize, &'a mut F) where F: Framebuffer;

impl<'a, F> PixelRef<'a, F> where F: Framebuffer {
    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <<F as Framebuffer>::Attachments as Attachments>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }
}

impl<'a, F> PixelMut<'a, F> where F: Framebuffer {
    /// Get the pixel
    #[inline]
    pub fn get(&self) -> <<F as Framebuffer>::Attachments as Attachments>::Color {
        unsafe { self.1.get_pixel_unchecked(self.0) }
    }

    /// Set the pixel
    #[inline]
    pub fn set(&mut self, color: <<F as Framebuffer>::Attachments as Attachments>::Color) {
        unsafe { self.1.set_pixel_unchecked(self.0, color) }
    }

    #[inline]
    pub fn into_ref(self) -> PixelRef<'a, F> {
        PixelRef(self.0, self.1)
    }
}

impl<'a, F> From<PixelMut<'a, F>> for PixelRef<'a, F> where F: Framebuffer {
    #[inline]
    fn from(pixel: PixelMut<'a, F>) -> PixelRef<'a, F> { pixel.into_ref() }
}

pub trait Framebuffer: Sized + 'static {
    /// Associated type for the framebuffer attachments
    type Attachments: Attachments;

    /// Returns the dimensions of the framebuffer
    fn dimensions(&self) -> Dimensions;

    /// Clears the framebuffer with the given color, and sets any depth or stencil buffers back to their default values.
    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color);

    /// Get pixel value without checking bounds.
    ///
    /// WARNING: This might segfault on an invalid index. Please use `pixel_ref`/`pixel_mut` for checked pixel access
    unsafe fn get_pixel_unchecked(&self, index: usize) -> <Self::Attachments as Attachments>::Color;

    /// Set pixel value without checking bounds.
    ///
    /// WARNING: This might segfault on an invalid index. Please use `pixel_ref`/`pixel_mut` for checked pixel access
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: <Self::Attachments as Attachments>::Color);

    /// Get a "reference" to the pixel at the given coordinate. Throws `RenderError::InvalidPixelCoordinate` on invalid pixel coordinates.
    #[inline]
    fn pixel_ref<'a>(&'a self, coord: Coordinate) -> RenderResult<PixelRef<'a, Self>> {
        self.dimensions().check_valid(coord).map(|_| {
            PixelRef(coord.into_index(), self)
        })
    }

    /// Get a mutable "reference" to the pixel at the given coordinate. Throws `RenderError::InvalidPixelCoordinate` on invalid pixel coordinates.
    #[inline]
    fn pixel_mut<'a>(&'a mut self, coord: Coordinate) -> RenderResult<PixelMut<'a, Self>> {
        self.dimensions().check_valid(coord).map(move |_| {
            PixelMut(coord.into_index(), self)
        })
    }
}

/// Renderbuffer framebuffer with interleaved attachments, allowing for more cache locality but
/// it cannot be re-used later as a texture without copying the attachments out.
pub struct RenderBuffer<A: Attachments> {
    dimensions: Dimensions,
    stencil: <A::Stencil as Stencil>::Config,
    /// Interlaced framebuffer for more cache-friendly access
    pub ( crate ) buffer: Vec<(A::Color, A::Depth, <A::Stencil as Stencil>::Type)>,
}

impl<A> Framebuffer for RenderBuffer<A> where A: Attachments {
    type Attachments = A;

    #[inline]
    fn dimensions(&self) -> Dimensions { self.dimensions }

    fn clear(&mut self, color: <Self::Attachments as Attachments>::Color) {
        for mut a in &mut self.buffer {
            *a = (color, <A::Depth as Depth>::far(), Default::default());
        }
    }

    #[inline]
    unsafe fn get_pixel_unchecked(&self, index: usize) -> <Self::Attachments as Attachments>::Color {
        self.buffer.get_unchecked(index).0
    }

    #[inline]
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: <Self::Attachments as Attachments>::Color) {
        self.buffer.get_unchecked_mut(index).0 = color;
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
                let len = dimensions.pixels();

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
    use attachments::color::predefined::formats::RGBAf32Color;

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