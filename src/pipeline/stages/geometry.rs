use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::Deref;

use rayon;
use rayon::prelude::*;

use smallvec::SmallVec;

use ::primitive::{Primitive, PrimitiveRef, Point, Line, Triangle};
use ::mesh::{Vertex, Mesh};
use ::geometry::{ClipVertex, ALL_CLIPPING_PLANES};
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
    pub fn finish(self, viewport: (V::Scalar, V::Scalar)) -> FragmentShader<'a, P, V, T, K, ()> {
        let GeometryShader { pipeline, mesh, indexed_vertices, stencil_value, generated_primitives, .. } = self;

        let SeparablePrimitiveStorage { points, lines, tris } = generated_primitives;

        let (indexed_vertices, generated_primitives) = rayon::join(move || {
            indexed_vertices.map(|indexed_vertices| {
                indexed_vertices.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect()
            })
        }, move || {
            let (points, (lines, tris)) = rayon::join(
                move || { points.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect() },
                move || {
                    rayon::join(
                        move || { lines.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect() },
                        move || { tris.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect() })
                });

            SeparableScreenPrimitiveStorage { points, lines, tris }
        });

        FragmentShader {
            pipeline: pipeline,
            mesh: mesh,
            indexed_primitive: PhantomData,
            stencil_value,
            indexed_vertices: Arc::new(indexed_vertices),
            generated_primitives: Arc::new(generated_primitives),
            cull_faces: None,
            blend: (),
            antialiased_lines: false,
            tile_size: DEFAULT_TILE_SIZE,
        }
    }

    #[must_use]
    pub fn run<S>(self, geometry_shader: S) -> Self
        where S: for<'s, 'p> Fn(PrimitiveStorage<'s, V::Scalar, K>, PrimitiveRef<'p, V::Scalar, K>, &PipelineUniforms<P>) + Send + Sync + 'static {
        let GeometryShader { pipeline, mesh, indexed_vertices, stencil_value, generated_primitives, .. } = self;

        let replaced_primitives = {
            let uniforms = pipeline.uniforms();

            // Queue up Points
            let points = generated_primitives.points.par_chunks(1).map(Point::create_ref_from_vertices);
            // Queue up Lines
            let lines = generated_primitives.lines.par_chunks(2).map(Line::create_ref_from_vertices);
            // Queue up Triangles
            let tris = generated_primitives.lines.par_chunks(3).map(Triangle::create_ref_from_vertices);

            // Chain together generated primitive queues
            let generated_primitives = points.chain(lines).chain(tris);

            // Create fold() closure
            let folder = |mut storage: SeparablePrimitiveStorage<V::Scalar, K>,
                          primitive: PrimitiveRef<V::Scalar, K>| {
                // Run the geometry shader here
                geometry_shader(PrimitiveStorage { inner: &mut storage }, primitive, uniforms);
                storage
            };

            // Create reduce() closure
            let reducer = |mut storage_a: SeparablePrimitiveStorage<V::Scalar, K>,
                           mut storage_b: SeparablePrimitiveStorage<V::Scalar, K>| {
                storage_a.append(&mut storage_b);
                storage_a
            };

            if let Some(ref indexed_vertices) = indexed_vertices {
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
            }
        };

        GeometryShader {
            pipeline,
            mesh,
            indexed_primitive: PhantomData,
            stencil_value,
            indexed_vertices: None,
            generated_primitives: replaced_primitives.unwrap_or_else(|| {
                SeparablePrimitiveStorage::default()
            }),
        }
    }

    #[must_use]
    pub fn clip_primitives(self) -> Self where K: Clone + Interpolate {
        self.run(|mut storage, primitive, _| {
            match primitive {
                PrimitiveRef::Triangle { a, b, c } => {
                    // We expect most triangles will go unchanged,
                    // so stack allocate them if possible.
                    let mut polygon: SmallVec<[_; 3]> = SmallVec::new();

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
                            storage.emit_triangle(last.clone(),
                                                  polygon[i].clone(),
                                                  polygon[i + 1].clone());
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