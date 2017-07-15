//! The rendering pipeline

use std::sync::Arc;
use std::marker::PhantomData;

use scoped_threadpool::Pool;
use num_cpus::get as num_cpus;

use ::mesh::{Vertex, Mesh};
use ::primitive::Primitive;
use ::geometry::Dimensions;
use ::stencil::StencilConfig;
use ::framebuffer::Framebuffer;
use ::framebuffer::nullbuffer::NullFramebuffer;

pub mod storage;
pub mod types;
pub mod stages;

pub use self::storage::PrimitiveStorage;
pub use self::stages::{VertexShader, GeometryShader, FragmentShader};

use self::types::StencilValue;

/// Defines types and methods for pipeline objects
pub trait PipelineObject {
    /// The associated framebuffer type for the pipeline
    type Framebuffer: Framebuffer;
    /// The associated global uniforms type for the pipeline
    type Uniforms: Send + Sync;
    /// The associated stencil configuration type for the pipeline
    type StencilConfig: StencilConfig;

    fn threadpool(&self) -> &Pool;
    fn threadpool_mut(&mut self) -> &mut Pool;

    /// Returns a reference to the stencil configuration
    fn stencil_config(&self) -> &Self::StencilConfig;
    /// Returns a mutable reference to the stencil configuration
    fn stencil_config_mut(&mut self) -> &mut Self::StencilConfig;

    /// Returns a reference to the uniforms value
    fn uniforms(&self) -> &Self::Uniforms;
    /// Returns a mutable reference to the uniforms value
    fn uniforms_mut(&mut self) -> &mut Self::Uniforms;

    /// Returns a reference to the framebuffer
    fn framebuffer(&self) -> &Self::Framebuffer;
    /// Returns a mutable reference to the framebuffer
    fn framebuffer_mut(&mut self) -> &mut Self::Framebuffer;

    #[inline]
    fn all_mut(&mut self) -> (&Self::Uniforms, &mut Self::Framebuffer, &mut Pool);
}

/// Starting point for the rendering pipeline.
///
/// By itself, it only holds the framebuffer and global uniforms,
/// but it spawns the first shader stage using those.
pub struct Pipeline<U, F, S = ()> {
    framebuffer: F,
    uniforms: U,
    stencil_config: S,
    threadpool: Pool,
}

impl<U, F, S> PipelineObject for Pipeline<U, F, S> where U: Send + Sync,
                                                         F: Framebuffer,
                                                         S: StencilConfig {
    type Framebuffer = F;
    type Uniforms = U;
    type StencilConfig = S;

    #[inline]
    fn threadpool(&self) -> &Pool { &self.threadpool }

    #[inline]
    fn threadpool_mut(&mut self) -> &mut Pool { &mut self.threadpool }

    #[inline]
    fn stencil_config(&self) -> &Self::StencilConfig {
        &self.stencil_config
    }

    #[inline]
    fn stencil_config_mut(&mut self) -> &mut Self::StencilConfig {
        &mut self.stencil_config
    }

    #[inline]
    fn uniforms(&self) -> &Self::Uniforms { &self.uniforms }
    #[inline]
    fn uniforms_mut(&mut self) -> &mut Self::Uniforms { &mut self.uniforms }

    #[inline]
    fn framebuffer(&self) -> &Self::Framebuffer { &self.framebuffer }
    #[inline]
    fn framebuffer_mut(&mut self) -> &mut Self::Framebuffer { &mut self.framebuffer }

    #[inline]
    fn all_mut(&mut self) -> (&Self::Uniforms, &mut Self::Framebuffer, &mut Pool) {
        (&self.uniforms, &mut self.framebuffer, &mut self.threadpool)
    }
}

impl<U, S> Pipeline<U, NullFramebuffer, S> where U: Send + Sync, S: StencilConfig {
    /// Create a new rendering pipeline instance with a `NullFramebuffer`.
    ///
    /// Use `from_framebuffer` or `with_framebuffer` to set the desired framebuffer for rendering.
    pub fn new(uniforms: U) -> Pipeline<U, NullFramebuffer, S> {
        Pipeline {
            framebuffer: NullFramebuffer::new(),
            uniforms,
            stencil_config: Default::default(),
            threadpool: Pool::new(num_cpus() as u32)
        }
    }

    /// Create a new pipeline from the given uniforms and framebuffer
    pub fn from_framebuffer<F>(framebuffer: F, uniforms: U) -> Pipeline<U, F, S> where F: Framebuffer {
        Self::new(uniforms).with_framebuffer(framebuffer)
    }

    /// Convert one pipeline into another with the given framebuffer,
    /// discarding the old framebuffer.
    pub fn with_framebuffer<F>(self, framebuffer: F) -> Pipeline<U, F, S> where F: Framebuffer {
        let Dimensions { width, height } = framebuffer.dimensions();

        assert!(width > 0, "Framebuffer must have a non-zero width");
        assert!(height > 0, "Framebuffer must have a non-zero height");

        let Pipeline { uniforms, threadpool, .. } = self;

        Pipeline {
            framebuffer,
            uniforms,
            stencil_config: Default::default(),
            threadpool,
        }
    }
}

impl<U, F, S> Pipeline<U, F, S> where Self: PipelineObject {
    /// Start the shading pipeline for a given mesh, with an optional stencil value for the mesh.
    #[must_use]
    pub fn render_mesh<T, V>(&mut self, primitive: T, mesh: Arc<Mesh<V>>, stencil: Option<StencilValue<Self>>) -> VertexShader<Self, V, T>
        where T: Primitive, V: Vertex {
        assert_eq!(mesh.indices.len() % T::num_vertices(), 0);

        // We only needed the type information,
        // so just throw away the empty object passed in
        drop(primitive);

        VertexShader { pipeline: self, mesh, stencil_value: stencil.unwrap_or_default(), indexed_primitive: PhantomData }
    }
}