/*

/// Geometry shader stage
///
/// The geometry shader can edit and generate new vertices from the output of the vertex shader.
///
/// Examples of geometry shader usage are crude tessellation, vertex displacement,
/// and geometry visualisations like normal vector lines.
///
/// The geometry shader can be ran multiple times.
pub struct GeometryShader<'a, T, V, U: 'a, K, P> where P: Pixel, T: Primitive {
    framebuffer: &'a mut FrameBuffer<P>,
    uniforms: &'a U,
    mesh: Arc<Mesh<V>>,
    indexed_primitive: PhantomData<T>,
    indexed_vertices: Option<Vec<ClipVertex<K>>>,
    generated_primitives: SeparablePrimitiveStorage<K>,
}


impl<'a, T, V, U: 'a, K, P> GeometryShader<'a, T, V, U, K, P> where T: Primitive,
                                                                    V: Send + Sync,
                                                                    U: Send + Sync,
                                                                    K: Send + Sync,
                                                                    P: Pixel {
    #[must_use]
    pub fn duplicate<'b>(&'b mut self) -> GeometryShader<'b, T, V, U, K, P> where 'a: 'b, K: Clone {
        /// Duplicate the geometry shader, and copies any processed geometry.
        ///
        /// Geometry are not synced between duplicated geometry shaders.
        GeometryShader {
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh.clone(),
            indexed_primitive: self.indexed_primitive,
            indexed_vertices: self.indexed_vertices.clone(),
            generated_primitives: self.generated_primitives.clone(),
        }
    }

    #[must_use]
    pub fn finish(self, viewport: (f32, f32)) -> FragmentShader<'a, T, V, U, K, P, ()> {
        let GeometryShader { framebuffer, uniforms, mesh, indexed_primitive, indexed_vertices, generated_primitives } = self;

        let SeparablePrimitiveStorage { points, lines, tris } = generated_primitives;

        let (indexed_vertices, generated_primitives) = rayon::join(move || {
            indexed_vertices.map(|indexed_vertices| {
                indexed_vertices.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect()
            })
        }, move || {
            let (points, (lines, tris)) = rayon::join(|| {
                points.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect()
            }, || {
                rayon::join(|| {
                    lines.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect()
                }, || {
                    tris.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect()
                })
            });

            SeparableScreenPrimitiveStorage { points, lines, tris }
        });

        FragmentShader {
            framebuffer: framebuffer,
            uniforms: uniforms,
            mesh: mesh,
            indexed_primitive: indexed_primitive,
            indexed_vertices: Arc::new(indexed_vertices),
            generated_primitives: Arc::new(generated_primitives),
            cull_faces: None,
            blend: (),
            antialiased_lines: false,
            tile_size: DEFAULT_TILE_SIZE
        }
    }

    #[must_use]
    pub fn replace<S>(self, geometry_shader: S) -> Self
        where S: for<'s, 'p> Fn(PrimitiveStorage<'s, K>, PrimitiveRef<'p, K>, &U) + Send + Sync + 'static {
        let GeometryShader { uniforms, framebuffer, mesh, indexed_vertices, generated_primitives, .. } = self;

        // Queue up Points
        let points = generated_primitives.points.par_chunks(1).map(Point::create_ref_from_vertices);
        // Queue up Lines
        let lines = generated_primitives.lines.par_chunks(2).map(Line::create_ref_from_vertices);
        // Queue up Triangles
        let tris = generated_primitives.lines.par_chunks(3).map(Triangle::create_ref_from_vertices);

        // Chain together generated primitive queues
        let generated_primitives = points.chain(lines).chain(tris);

        // Create fold() closure
        let folder = |mut storage: SeparablePrimitiveStorage<K>,
                      primitive: PrimitiveRef<K>| {
            // Run the geometry shader here
            geometry_shader(PrimitiveStorage { inner: &mut storage }, primitive, uniforms);
            storage
        };

        // Create reduce() closure
        let reducer = |mut storage_a: SeparablePrimitiveStorage<K>,
                       mut storage_b: SeparablePrimitiveStorage<K>| {
            storage_a.append(&mut storage_b);
            storage_a
        };

        let replaced_primitives = if let Some(ref indexed_vertices) = indexed_vertices {
            let num_vertices = <T as Primitive>::num_vertices();

            let indexed = mesh.indices.par_chunks(num_vertices).map(|indices| {
                <T as Primitive>::create_ref_from_indexed_vertices(&indexed_vertices, indices)
            });

            // Just chain together the indexed primitives and generated primitives
            indexed.chain(generated_primitives).with_min_len(1024)
                   .fold(|| SeparablePrimitiveStorage::default(), folder)
                   .reduce_with(reducer)
        } else {
            generated_primitives.with_min_len(1024)
                                .fold(|| SeparablePrimitiveStorage::default(), folder)
                                .reduce_with(reducer)
        };

        GeometryShader {
            uniforms,
            framebuffer,
            mesh,
            indexed_primitive: PhantomData,
            indexed_vertices: None,
            generated_primitives: replaced_primitives.unwrap_or_else(|| SeparablePrimitiveStorage::default()),
        }
    }

    #[must_use]
    pub fn clip_primitives(self) -> Self where K: Clone + Interpolate {
        self.replace(|mut storage, primitive, _| {
            match primitive {
                PrimitiveRef::Triangle { a, b, c } => {
                    // We expect most triangles will go unchanged
                    let mut polygon = Vec::with_capacity(3);

                    for &(s, p) in &[(a, b), (b, c), (c, a)] {
                        for plane in &ALL_CLIPPING_PLANES {
                            let s_in = plane.has_inside(s);
                            let p_in = plane.has_inside(p);

                            if s_in != p_in {
                                polygon.push(plane.intersect(s, p));
                            }

                            if p_in {
                                polygon.push(p.clone());
                            }
                        }
                    }

                    if polygon.len() == 3 {
                        storage.inner.tris.append(&mut polygon);
                    } else if polygon.len() > 3 {
                        let last = polygon.last().unwrap();

                        for i in 0..polygon.len() - 2 {
                            storage.emit_triangle(last.clone(), polygon[i].clone(), polygon[i + 1].clone());
                        }
                    }
                }
                _ => storage.re_emit(primitive)
            }
        })
    }
}
*/