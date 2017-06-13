pub mod storage;
pub mod stages;

pub use self::storage::PrimitiveStorage;

/*
/// Starting point for the rendering pipeline.
///
/// By itself, it only holds the framebuffer and global uniforms,
/// but it spawns the first shader stage using those.
pub struct Pipeline<U, P> where P: Pixel {
    framebuffer: FrameBuffer<P>,
    uniforms: U,
}

impl<U, P> Pipeline<U, P> where U: Send + Sync,
                                P: Pixel {
    /// Create a new rendering pipeline instance
    pub fn new(framebuffer: FrameBuffer<P>, uniforms: U) -> Pipeline<U, P> {
        assert!(framebuffer.width() > 0, "Framebuffer must have a non-zero width");
        assert!(framebuffer.height() > 0, "Framebuffer must have a non-zero height");

        Pipeline {
            framebuffer: framebuffer,
            uniforms: uniforms,
        }
    }

    /// Start the shading pipeline for a given mesh
    #[must_use]
    pub fn render_mesh<T, V>(&mut self, mesh: Arc<Mesh<V>>) -> VertexShader<T, V, U, P> where T: Primitive,
                                                                                              V: Send + Sync {
        VertexShader {
            mesh: mesh,
            indexed_primitive: PhantomData,
            uniforms: &self.uniforms,
            framebuffer: &mut self.framebuffer,
        }
    }

    /// Returns a reference to the uniforms value
    pub fn uniforms(&self) -> &U { &self.uniforms }
    /// Returns a mutable reference to the uniforms value
    pub fn uniforms_mut(&mut self) -> &mut U { &mut self.uniforms }

    /// Returns a reference to the framebuffer
    pub fn framebuffer(&self) -> &FrameBuffer<P> { &self.framebuffer }
    /// Returns a mutable reference to the framebuffer
    pub fn framebuffer_mut(&mut self) -> &mut FrameBuffer<P> { &mut self.framebuffer }
}
*/