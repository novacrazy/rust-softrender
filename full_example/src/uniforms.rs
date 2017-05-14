use nalgebra::{Vector2, Vector4, Matrix4};

use ::light::Light;

/// Define global uniforms. These don't need to be interpolated, so they can just be any type.
#[derive(Debug, Clone)]
pub struct GlobalUniforms {
    pub camera: Vector4<f32>,
    pub model: Matrix4<f32>,
    /// the inverse transpose of the model matrix is
    /// multiplied by the normal vector to get the correct value
    pub mit: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub lights: Vec<Light>
}

declare_uniforms! {
    /// Uniforms which can be passed through the shader pipeline and interpolated on a triangle
    #[derive(Debug, Clone, Copy)]
    pub struct Uniforms {
        /// Position in world-space
        pub position: Vector4<f32>,
        /// Surface normal in world-space
        pub normal: Vector4<f32>,
        // uv-coordinates for textures
        pub uv: Vector2<f32>,
    }
}