/*
/// Fragment shader stage.
///
/// The fragment shader is responsible for determining the color of pixels where the underlying geometry has been projected onto.
/// Usually this is individual triangles that are rasterized and shaded by the fragment shader, but it also supports point-cloud
/// and lines (pairs of vertices considered as endpoints for lines).
///
/// The fragment shader runs several tests before executing the given shader program, including a depth test.
/// If the depth of the geometry (from the camera), is farther away than geometry that has already been rendered,
/// the shader program isn't run at all, since it wouldn't be visible anyway. Additionally,
/// if the geometry is nearer than an existing fragment, the existing fragment is overwritten.
///
/// Uniforms passed from the vertex shader are interpolating inside the triangles using Interpolate interpolation,
/// which is why it must satisfy the [`Interpolate`](../uniform/trait.Interpolate.html) trait, which can be automatically implemented for many types using the
/// `declare_uniforms!` macro. See the documentation on that for more information on how to use it.
pub struct FragmentShader<'a, T, V, U: 'a, K, P, B = ()> where P: Pixel, T: Primitive {
    framebuffer: &'a mut FrameBuffer<P>,
    uniforms: &'a U,
    mesh: Arc<Mesh<V>>,
    indexed_primitive: PhantomData<T>,
    indexed_vertices: Arc<Option<Vec<ScreenVertex<K>>>>,
    generated_primitives: Arc<SeparableScreenPrimitiveStorage<K>>,
    cull_faces: Option<FaceWinding>,
    blend: B,
    antialiased_lines: bool,
    tile_size: (u32, u32),
}

pub const DEFAULT_TILE_SIZE: (u32, u32) = (256, 256);


/// Fragment returned by the fragment shader, which can either be a color
/// value for the pixel or a discard flag to skip that fragment altogether.
#[derive(Debug, Clone, Copy)]
pub enum Fragment<P> where P: Pixel {
    /// Discard the fragment altogether, as if it was never there.
    Discard,
    /// Desired color for the pixel
    Color(P)
}

impl<'a, T, V, U: 'a, K, P, B> Deref for FragmentShader<'a, T, V, U, K, P, B> where T: Primitive,
                                                                                    P: Pixel,
                                                                                    B: Blend<P> {
    type Target = B;
    fn deref(&self) -> &B { &self.blend }
}

impl<'a, T, V, U: 'a, K, P, B> DerefMut for FragmentShader<'a, T, V, U, K, P, B> where T: Primitive,
                                                                                       P: Pixel,
                                                                                       B: Blend<P> {
    fn deref_mut(&mut self) -> &mut B { &mut self.blend }
}

impl<'a, T, V, U, K, P, B> FragmentShader<'a, T, V, U, K, P, B> where T: Primitive,
                                                                      P: Pixel {
    /// Cull faces based on winding order. For more information on how and why this works,
    /// check out the documentation for the [`FaceWinding`](../geometry/enum.FaceWinding.html) enum.
    #[inline(always)]
    pub fn cull_faces(&mut self, cull: Option<FaceWinding>) {
        self.cull_faces = cull;
    }

    /// Enables drawing antialiased lines for `Line` primitives
    /// primitives using Xiaolin Wu's algorithm
    #[inline(always)]
    pub fn antialiased_lines(&mut self, enable: bool) {
        self.antialiased_lines = enable;
    }
}

impl<'a, T, V, U, K, P, O> FragmentShader<'a, T, V, U, K, P, O> where T: Primitive,
                                                                      P: Pixel {
    #[must_use]
    pub fn with_blend<B>(self, blend: B) -> FragmentShader<'a, T, V, U, K, P, B> where B: Blend<P> {
        FragmentShader {
            blend: blend,
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh,
            indexed_primitive: PhantomData,
            indexed_vertices: self.indexed_vertices,
            generated_primitives: self.generated_primitives,
            cull_faces: self.cull_faces,
            antialiased_lines: self.antialiased_lines,
            tile_size: self.tile_size,
        }
    }

    #[must_use]
    pub fn with_default_blend<B>(self) -> FragmentShader<'a, T, V, U, K, P, B> where B: Blend<P> + Default {
        self.with_blend(B::default())
    }
}

impl<'a, T, V, U: 'a, K, P, B> FragmentShader<'a, T, V, U, K, P, B> where T: Primitive,
                                                                          V: Send + Sync,
                                                                          U: Send + Sync,
                                                                          K: Send + Sync + Interpolate,
                                                                          P: Pixel, B: Blend<P> {
    /// Duplicates all references to internal state to return a cloned fragment shader,
    /// which can be used to efficiently render the same geometry with different
    /// rasterization methods in quick succession.
    #[must_use]
    pub fn duplicate<'b>(&'b mut self) -> FragmentShader<'b, T, V, U, K, P, B> where 'a: 'b,
                                                                                     B: Clone {
        FragmentShader {
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh.clone(),
            indexed_primitive: PhantomData,
            indexed_vertices: self.indexed_vertices.clone(),
            generated_primitives: self.generated_primitives.clone(),
            cull_faces: self.cull_faces.clone(),
            blend: self.blend.clone(),
            antialiased_lines: self.antialiased_lines,
            tile_size: self.tile_size,
        }
    }

    #[must_use]
    pub fn run<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {}
}

*/