/*

/// Vertex shader stage.
///
/// The vertex shader is responsible for transforming all mesh vertices into a form which can be presented on screen (more or less),
/// which usually involved transforming object-space coordinates to world-space, then to camera-space, then finally to projection/clip-space,
/// at which point it and any uniforms are passed back and sent to the fragment shader.
///
/// For a full example of how this works, see the documentation on the `run` method below.
///
/// The vertex shader holds a reference to the pipeline framebuffer and global uniforms,
/// and for the given mesh given to it when created.
/// These cannot be modified while the vertex shader exists.
pub struct VertexShader<'a, T, V, U: 'a, P> where P: Pixel, T: Primitive {
    framebuffer: &'a mut FrameBuffer<P>,
    uniforms: &'a U,
    mesh: Arc<Mesh<V>>,
    indexed_primitive: PhantomData<T>,
}


impl<'a, T, V, U: 'a, P> VertexShader<'a, T, V, U, P> where T: Primitive,
                                                            V: Send + Sync,
                                                            U: Send + Sync,
                                                            P: Pixel {
    /// Duplicates all references to internal state to return a cloned vertex shader,
    /// though since the vertex shader itself has very little internal state at this point,
    /// it's not that useful.
    #[must_use]
    pub fn duplicate<'b>(&'b mut self) -> VertexShader<'b, T, V, U, P> where 'a: 'b {
        VertexShader {
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh.clone(),
            indexed_primitive: PhantomData,
        }
    }

    /// Executes the vertex shader on every vertex in the mesh,
    /// (hopefully) returning a `ClipVertex` with the transformed vertex in clip-space
    /// and any uniforms to be passed into the fragment shader.
    ///
    /// In case you don't want to research what clip-space is, it's basically the output of the projection transformation,
    /// so in your vertex shader you'd have something like:
    ///
    /// ```ignore
    /// let fragment_shader = vertex_shader.run(|vertex, global_uniforms| {
    ///     let GlobalUniforms { ref view, ref projection, ref model, ref model_inverse_transpose, .. } = *global_uniforms;
    ///     let VertexData { normal, uv } = vertex.vertex_data;
    ///
    ///     // Transform vertex position to world-space
    ///     let world_position = model * vertex.position.to_homogeneous();
    ///
    ///     // Transform normal to world-space
    ///     let normal = (model_inverse_transpose * normal.to_homogeneous()).normalize();
    ///
    ///     // Transform vertex position to clip-space (projection-space)
    ///     let clip_position = projection * view * world_position;
    ///
    ///     // Return the clip-space position and any uniforms to interpolate and pass into the fragment shader
    ///     ClipVertex::new(clip_position, Uniforms {
    ///         position: world_position,
    ///         normal: normal,
    ///         uv: uv,
    ///     })
    /// });
    /// ```
    ///
    /// where `GlobalUniforms`, `VertexData` and `Uniforms` are data structures defined by you.
    ///
    /// See the [`full_example`](https://github.com/novacrazy/rust-softrender/tree/master/full_example) project for this in action.
    #[must_use]
    pub fn run<S, K>(self, vertex_shader: S) -> GeometryShader<'a, T, V, U, K, P> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Send + Sync,
                                                                                        K: Send + Sync + Interpolate {
        let VertexShader {
            framebuffer,
            uniforms,
            mesh,
            ..
        } = self;

        let indexed_vertices = mesh.vertices.par_iter().map(|vertex| {
            vertex_shader(vertex, uniforms)
        }).collect();

        GeometryShader {
            framebuffer: framebuffer,
            uniforms: uniforms,
            mesh: mesh,
            indexed_primitive: PhantomData,
            indexed_vertices: Some(indexed_vertices),
            generated_primitives: SeparablePrimitiveStorage::default(),
        }
    }

    /// Same as `run`, but skips the geometry shader stage.
    ///
    /// This pathway does not do any clipping, so beware of that when rendering. However,
    /// it is the fastest path, so the tradeoff may be acceptable for some use cases.
    #[must_use]
    pub fn run_to_fragment<S, K>(self, viewport: (f32, f32), vertex_shader: S) -> FragmentShader<'a, T, V, U, K, P, ()> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                                                              K: Send + Sync + Interpolate {
        let VertexShader {
            framebuffer,
            uniforms,
            mesh,
            ..
        } = self;

        let indexed_vertices = mesh.vertices.par_iter().map(|vertex| {
            vertex_shader(vertex, uniforms)
                .normalize(viewport)
        }).collect();

        FragmentShader {
            framebuffer: framebuffer,
            uniforms: uniforms,
            mesh: mesh,
            indexed_primitive: PhantomData,
            indexed_vertices: Arc::new(Some(indexed_vertices)),
            generated_primitives: Arc::new(SeparableScreenPrimitiveStorage::default()),
            cull_faces: None,
            blend: (),
            antialiased_lines: false,
            tile_size: DEFAULT_TILE_SIZE
        }
    }
}

*/