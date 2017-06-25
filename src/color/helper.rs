//! Helper traits

/// Helper trait to aid in alpha multiplication,
/// primarily of integer types where direct multiplication is invalid.
///
/// For example, using an RGBA u8 Color type, where the color black is defined as
/// `(0, 0, 0, 256)`, multiplying that by an alpha of `128`
/// should give `(0, 0, 0, 256 * (128 / 256))`, not `(0, 0, 0, 256 * 128)`.
///
/// Additionally, for signed integer types, negative alpha values are undefined and thus return zero.
///
/// So casting to float and dividing by the maximum integer value is required to get the correct value.
/// This trait helps with that for all built-in primitive types.
pub trait AlphaMultiply {
    /// Multiply a channel by the alpha value.
    ///
    /// This is usually only applicable to another alpha channel.
    fn mul_alpha(channel: Self, alpha: Self) -> Self;
}

impl AlphaMultiply for f32 {
    #[inline(always)]
    fn mul_alpha(channel: f32, alpha: f32) -> f32 { channel * alpha }
}

impl AlphaMultiply for f64 {
    #[inline(always)]
    fn mul_alpha(channel: f64, alpha: f64) -> f64 { channel * alpha }
}

macro_rules! unsigned_integer_alpha_helpers {
    ($($t:ident -> $f:ident,)+) => {
        $(
            impl AlphaMultiply for $t {
                #[inline]
                fn mul_alpha(channel: $t, alpha: $t) -> $t {
                    (channel as $f * (alpha as $f / (::std::$t::MAX as $f))) as $t
                }
            }
        )+
    }
}

macro_rules! signed_integer_alpha_helpers {
    ($($t:ident -> $f:ident,)+) => {
        $(
            impl AlphaMultiply for $t {
                #[inline]
                fn mul_alpha(channel: $t, alpha: $t) -> $t {
                    if alpha < 0 { 0 } else {
                        (channel as $f * (alpha as $f / (::std::$t::MAX as $f))) as $t
                    }
                }
            }
        )+
    }
}

unsigned_integer_alpha_helpers!(
    u8 -> f32,
    u16 -> f32,
    u32 -> f32,
    u64 -> f64,
    usize -> f64,
);

signed_integer_alpha_helpers!(
    i8 -> f32,
    i16 -> f32,
    i32 -> f32,
    i64 -> f64,
    isize -> f64,
);