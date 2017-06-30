//! The rendering pipeline

use std::sync::Arc;
use std::marker::PhantomData;

use ::mesh::{Vertex, Mesh};
use ::primitive::Primitive;
use ::geometry::Dimensions;
use ::framebuffer::Framebuffer;
use ::framebuffer::nullbuffer::NullFramebuffer;

pub mod storage;
pub mod types;
pub mod stages;

pub use self::storage::PrimitiveStorage;

pub use self::stages::{VertexShader, GeometryShader, FragmentShader};

/// Defines types and methods for pipeline objects
pub trait PipelineObject {
    /// The associated framebuffer type for the pipeline
    type Framebuffer: Framebuffer;
    /// The associated global uniforms type for the pipeline
    type Uniforms: Send + Sync;

    /// Returns a reference to the uniforms value
    fn uniforms(&self) -> &Self::Uniforms;
    /// Returns a mutable reference to the uniforms value
    fn uniforms_mut(&mut self) -> &mut Self::Uniforms;

    /// Returns a reference to the framebuffer
    fn framebuffer(&self) -> &Self::Framebuffer;
    /// Returns a mutable reference to the framebuffer
    fn framebuffer_mut(&mut self) -> &mut Self::Framebuffer;
}

/// Starting point for the rendering pipeline.
///
/// By itself, it only holds the framebuffer and global uniforms,
/// but it spawns the first shader stage using those.
#[derive(Clone)]
pub struct Pipeline<U, F> {
    framebuffer: F,
    uniforms: U,
}

impl<U, F> PipelineObject for Pipeline<U, F> where U: Send + Sync, F: Framebuffer {
    type Framebuffer = F;
    type Uniforms = U;

    /// Returns a reference to the uniforms value
    #[inline]
    fn uniforms(&self) -> &Self::Uniforms { &self.uniforms }
    /// Returns a mutable reference to the uniforms value
    #[inline]
    fn uniforms_mut(&mut self) -> &mut Self::Uniforms { &mut self.uniforms }

    /// Returns a reference to the framebuffer
    #[inline]
    fn framebuffer(&self) -> &Self::Framebuffer { &self.framebuffer }
    /// Returns a mutable reference to the framebuffer
    #[inline]
    fn framebuffer_mut(&mut self) -> &mut Self::Framebuffer { &mut self.framebuffer }
}

impl<U> Pipeline<U, NullFramebuffer> where U: Send + Sync {
    /// Create a new rendering pipeline instance with a `NullFramebuffer`.
    ///
    /// Use `from_framebuffer` or `with_framebuffer` to set the desired framebuffer for rendering.
    pub fn new(uniforms: U) -> Pipeline<U, NullFramebuffer> {
        Pipeline { framebuffer: NullFramebuffer::new(), uniforms }
    }

    /// Create a new pipeline from the given uniforms and framebuffer
    pub fn from_framebuffer<F>(framebuffer: F, uniforms: U) -> Pipeline<U, F> where F: Framebuffer {
        Self::new(uniforms).with_framebuffer(framebuffer)
    }

    /// Convert one pipeline into another with the given framebuffer,
    /// discarding the old framebuffer.
    pub fn with_framebuffer<F>(self, framebuffer: F) -> Pipeline<U, F> where F: Framebuffer {
        let Dimensions { width, height } = framebuffer.dimensions();

        assert!(width > 0, "Framebuffer must have a non-zero width");
        assert!(height > 0, "Framebuffer must have a non-zero height");

        Pipeline {
            framebuffer,
            uniforms: self.uniforms
        }
    }
}

impl<U, F> Pipeline<U, F> where Self: PipelineObject {
    /// Start the shading pipeline for a given mesh
    #[must_use]
    pub fn render_mesh<T, V>(&mut self, primitive: T, mesh: Arc<Mesh<V>>) -> VertexShader<Self, V, T> where T: Primitive,
                                                                                                            V: Vertex {
        // We only needed the type information,
        // so just throw away the empty object passed in
        drop(primitive);

        VertexShader { pipeline: self, mesh, indexed_primitive: PhantomData }
    }
}