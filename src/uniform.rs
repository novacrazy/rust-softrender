//! Types, traits and macros for uniform variables
//!
//! Currently, the `Interpolate` trait is implemented for `f32` and nalgebra
//! matrices (including vectors), points, translations, rotations, and Quaternions.
//!
//! It can be implemented automatically for your uniforms structures by using the [`declare_uniforms!`](../../macro.declare_uniforms.html) macro.

/// Declares a structure and implements the [`Interpolate`](render/uniform/trait.Interpolate.html) trait for it by delegating the trait to each member.
///
/// So, for example, this:
///
/// ```ignore
/// declare_uniforms!(
///     pub struct MyUniforms {
///         /// Position in world-space
///         pub position: Vector4<f32>,
///         pub normal: Vector4<f32>,
///         pub uv: Vector2<f32>,
///     }
/// );
/// ```
///
/// becomes:
///
/// ```ignore
/// pub struct MyUniforms {
///     /// Position in world-space
///     pub position: Vector4<f32>,
///     pub normal: Vector4<f32>,
///     pub uv: Vector2<f32>,
/// }
///
/// impl Interpolate for MyUniforms {
///     fn interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
///         MyUniforms {
///             position: Interpolate::barycentric_interpolate(u, &ux.position, v, &vx.position, w, &wx.position),
///             normal: Interpolate::barycentric_interpolate(u, &ux.normal, v, &vx.normal, w, &wx.normal),
///             uv: Interpolate::barycentric_interpolate(u, &ux.uv, v, &vx.uv, w, &wx.uv),
///         }
///     }
/// }
/// ```
///
/// note that the `u` and `v` in the `Interpolate::barycentric_interpolate` arguments are mostly unrelated to the `uv` normal. They're both Interpolate coordinates,
/// but for different things.
///
/// For now, the struct itself must be `pub` and all the members must be `pub`, but hopefully that can change in the future.
#[macro_export]
macro_rules! declare_uniforms {
    ($(#[$($struct_attrs:tt)*])* pub struct $name:ident {
        $($(#[$($field_attrs:tt)*])* pub $field:ident: $t:ty,)*
    }) => {
        $(#[$($struct_attrs)*])*
        pub struct $name {
            $(
                $(#[$($field_attrs)*])*
                pub $field: $t
            ),*
        }

        impl $crate::Interpolate for $name {
            fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self {
                $name {
                    $(
                        $field: $crate::Interpolate::barycentric_interpolate(u, &ux.$field,
                                                                                     v, &vx.$field,
                                                                                     w, &wx.$field)
                    ),*
                }
            }

            fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
                $name {
                    $(
                          $field: $crate::Interpolate::linear_interpolate(t, &x1.$field, &x2.$field)
                    ),*
                }
            }
        }
    };
}