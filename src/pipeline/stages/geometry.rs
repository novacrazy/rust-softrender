use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{ptr, mem};

use smallvec::SmallVec;
use parking_lot::Mutex;

use ::parallel::{TrustedThreadSafe, CACHE_LINE_SIZE, Mapper};

use ::primitive::{Primitive, PrimitiveRef, Point, Line, Triangle};
use ::mesh::{Vertex, Mesh};
use ::geometry::{ClipVertex, Viewport, ScreenVertex, ALL_CLIPPING_PLANES, ClippingPlane};
use ::interpolate::Interpolate;
use ::pipeline::storage::{PrimitiveStorage, SeparablePrimitiveStorage, SeparableScreenPrimitiveStorage};
use ::pipeline::{PipelineObject, FragmentShader};
use ::pipeline::stages::fragment::DEFAULT_TILE_SIZE;

use ::pipeline::types::{PipelineUniforms, StencilValue};

/// Geometry shader stage
///
/// The geometry shader can edit and generate new vertices from the output of the vertex shader.
///
/// Examples of geometry shader usage are crude tessellation, vertex displacement,
/// and geometry visualisations like normal vector lines.
///
/// The geometry shader can be ran multiple times.
pub struct GeometryShader<'a, P: 'a, V: Vertex, T, K> where P: PipelineObject {
    pub ( in ::pipeline) pipeline: &'a mut P,
    pub ( in ::pipeline) mesh: Arc<Mesh<V>>,
    pub ( in ::pipeline) indexed_primitive: PhantomData<T>,
    pub ( in ::pipeline) stencil_value: StencilValue<P>,
    pub ( in ::pipeline) indexed_vertices: Option<Vec<ClipVertex<V::Scalar, K>>>,
    pub ( in ::pipeline) generated_primitives: SeparablePrimitiveStorage<V::Scalar, K>,
}

impl<'a, P: 'a, V, T, K> GeometryShader<'a, P, V, T, K> where P: PipelineObject, V: Vertex {
    /// Duplicate the geometry shader, and copies any processed geometry.
    ///
    /// Geometry are not synced between duplicated geometry shaders.
    #[must_use]
    pub fn duplicate<'b>(&'b mut self) -> GeometryShader<'b, P, V, T, K> where 'a: 'b, K: Clone {
        GeometryShader {
            pipeline: self.pipeline,
            mesh: self.mesh.clone(),
            indexed_primitive: PhantomData,
            stencil_value: self.stencil_value,
            indexed_vertices: self.indexed_vertices.clone(),
            generated_primitives: self.generated_primitives.clone(),
        }
    }
}

impl<'a, P: 'a, V, T, K> GeometryShader<'a, P, V, T, K> where P: PipelineObject,
                                                              V: Vertex,
                                                              T: Primitive,
                                                              K: Send + Sync + Interpolate {
    #[must_use]
    pub fn finish(self, viewport: Viewport<V::Scalar>) -> FragmentShader<'a, P, V, T, K, ()> {
        let GeometryShader { pipeline, mesh, indexed_vertices, stencil_value, generated_primitives, .. } = self;

        let SeparablePrimitiveStorage { mut points, mut lines, mut tris } = generated_primitives;

        let (indexed_screen_vertices, generated_primitives) = {
            let pool = pipeline.threadpool_mut();

            let thread_count = pool.thread_count();

            let point_mapper = Mapper::new(points.len());
            let line_mapper = Mapper::new(lines.len());
            let tri_mapper = Mapper::new(tris.len());

            let indexed_mapper = indexed_vertices.as_ref().map(|iv| {
                Mapper::new(iv.len())
            });

            pool.scoped(|scope| {
                for _ in 0..thread_count {
                    scope.execute(|| {
                        point_mapper.map_move(&points, |vertex| vertex.normalize(viewport));
                        line_mapper.map_move(&lines, |vertex| vertex.normalize(viewport));
                        tri_mapper.map_move(&tris, |vertex| vertex.normalize(viewport));

                        if let Some(ref indexed_mapper) = indexed_mapper {
                            if let Some(ref indexed_vertices) = indexed_vertices {
                                indexed_mapper.map_move(&indexed_vertices, |vertex| { vertex.normalize(viewport) })
                            }
                        }
                    });
                }
            });

            let storage = SeparableScreenPrimitiveStorage {
                points: point_mapper.into_target(),
                lines: line_mapper.into_target(),
                tris: tri_mapper.into_target(),
            };

            let indexed_vertices = indexed_mapper.map(|im| im.into_target());

            (indexed_vertices, storage)
        };

        // We used map_move, so the values have already been moved,
        // but we need to manually set the vectors to zero to prevent double-drops
        unsafe {
            points.set_len(0);
            lines.set_len(0);
            tris.set_len(0);
        }

        if let Some(mut indexed_vertices) = indexed_vertices {
            unsafe { indexed_vertices.set_len(0); }
        }

        FragmentShader {
            pipeline: pipeline,
            mesh: mesh,
            indexed_primitive: PhantomData,
            stencil_value,
            indexed_vertices: Arc::new(indexed_screen_vertices),
            generated_primitives: Arc::new(generated_primitives),
            cull_faces: None,
            blend: (),
            antialiased_lines: false,
            tile_size: DEFAULT_TILE_SIZE,
        }
    }

    #[must_use]
    pub fn run<S, Y>(self, geometry_shader: S) -> GeometryShader<'a, P, V, T, Y>
        where S: for<'s, 'p> Fn(PrimitiveStorage<'s, V::Scalar, Y>, PrimitiveRef<'p, V::Scalar, K>, &PipelineUniforms<P>) + Send + Sync + 'static,
              Y: Send + Sync + Interpolate {
        let GeometryShader { pipeline, mesh, indexed_vertices, stencil_value, generated_primitives, .. } = self;

        let replaced_primitives = {
            let SeparablePrimitiveStorage { ref points, ref lines, ref tris } = generated_primitives;

            let mut replaced_primitives_unmerged = {
                let (uniforms, _, pool) = pipeline.all_mut();

                let thread_count = pool.thread_count();

                let point_i = AtomicUsize::new(0);
                let line_i = AtomicUsize::new(0);
                let tri_i = AtomicUsize::new(0);
                let indexed_i = AtomicUsize::new(0);

                let replaced_primitives_unmerged = Mutex::new(Vec::with_capacity(pool.thread_count() as usize));

                pool.scoped(|scope| {
                    for _ in 0..thread_count {
                        scope.execute(|| {
                            let mut storage = SeparablePrimitiveStorage::default();

                            loop {
                                let i = point_i.fetch_add(Point::num_vertices(), Ordering::Relaxed);

                                if i < points.len() {
                                    geometry_shader(
                                        PrimitiveStorage { inner: &mut storage },
                                        Point::create_ref_from_vertices(&points[i..]),
                                        uniforms,
                                    );
                                } else {
                                    break;
                                }
                            }

                            loop {
                                let i = line_i.fetch_add(Line::num_vertices(), Ordering::Relaxed);

                                if i < lines.len() {
                                    geometry_shader(
                                        PrimitiveStorage { inner: &mut storage },
                                        Line::create_ref_from_vertices(&lines[i..]),
                                        uniforms,
                                    );
                                } else {
                                    break;
                                }
                            }

                            loop {
                                let i = tri_i.fetch_add(Triangle::num_vertices(), Ordering::Relaxed);

                                if i < tris.len() {
                                    geometry_shader(
                                        PrimitiveStorage { inner: &mut storage },
                                        Triangle::create_ref_from_vertices(&tris[i..]),
                                        uniforms,
                                    );
                                } else {
                                    break;
                                }
                            }

                            if let Some(ref indexed_vertices) = indexed_vertices {
                                let len = mesh.indices.len();

                                loop {
                                    let mut i = indexed_i.fetch_add(T::num_vertices(), Ordering::Relaxed);

                                    if i < len {
                                        geometry_shader(
                                            PrimitiveStorage { inner: &mut storage },
                                            T::create_ref_from_indexed_vertices(&indexed_vertices, &mesh.indices[i..]),
                                            uniforms,
                                        );
                                    } else {
                                        break;
                                    }
                                }
                            }

                            let mut replaced_primitives_unmerged = replaced_primitives_unmerged.lock();

                            replaced_primitives_unmerged.push(storage);
                        });
                    }
                });

                replaced_primitives_unmerged.into_inner()
            };

            let mut num_point_vertices = 0;
            let mut num_line_vertices = 0;
            let mut num_tri_vertices = 0;

            for v in &replaced_primitives_unmerged {
                num_point_vertices += v.points.len();
                num_line_vertices += v.lines.len();
                num_tri_vertices += v.tris.len();
            }

            let mut storage = SeparablePrimitiveStorage {
                points: Vec::with_capacity(num_point_vertices),
                lines: Vec::with_capacity(num_line_vertices),
                tris: Vec::with_capacity(num_tri_vertices),
            };

            for v in &mut replaced_primitives_unmerged {
                storage.append(v);
            }

            storage
        };

        GeometryShader {
            pipeline,
            mesh,
            indexed_primitive: PhantomData,
            stencil_value,
            indexed_vertices: None,
            generated_primitives: replaced_primitives,
        }
    }

    #[must_use]
    pub fn clip_primitives(self) -> Self where K: Clone + Interpolate {
        self.run(|mut storage, primitive, _| {
            match primitive {
                PrimitiveRef::Triangle { a, b, c } => {
                    // We expect most triangles will go unchanged,
                    // or only add a single extra vertex,
                    // so stack allocate them if possible.
                    let mut polygon: SmallVec<[_; 4]> = SmallVec::new();

                    for &(s, p) in &[(a, b), (b, c), (c, a)] {
                        for plane in &ALL_CLIPPING_PLANES {
                            let s_in = plane.has_inside(s);
                            let p_in = plane.has_inside(p);

                            // Edge intersects clipping plane
                            if s_in != p_in {
                                polygon.push(plane.intersect(s, p));
                            }

                            if p_in {
                                polygon.push(p.clone());
                            }
                        }
                    }

                    if polygon.len() == 3 {
                        storage.inner.tris.extend_from_slice(&polygon);
                    } else if polygon.len() > 3 {
                        let last = polygon.last().unwrap();

                        for i in 0..polygon.len() - 2 {
                            storage.emit_triangle(
                                last.clone(),
                                polygon[i].clone(),
                                polygon[i + 1].clone(),
                            );
                        }
                    }
                }
                PrimitiveRef::Line { start, end } => {
                    let mut start = start.clone();
                    let mut end = end.clone();

                    let mut intersections = 0;

                    for plane in &ALL_CLIPPING_PLANES {
                        let s_in = plane.has_inside(&start);
                        let p_in = plane.has_inside(&end);

                        if s_in != p_in {
                            let intersection = plane.intersect(&start, &end);

                            if s_in {
                                end = intersection;
                            } else if p_in {
                                start = intersection;
                            }

                            intersections += 1;
                        } else if !s_in { return; }

                        // A line segment can only intersect with two planes at a time,
                        // so skip the rest when two intersections are found.
                        if intersections > 2 { break; }
                    }

                    storage.emit_line(start, end)
                }
                PrimitiveRef::Point(point) => {
                    if ALL_CLIPPING_PLANES.iter().all(|plane| plane.has_inside(&point)) {
                        storage.emit_point(point.clone());
                    }
                }
            }
        })
    }
}