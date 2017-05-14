//! Rendering pipeline implementation

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use rayon::current_num_threads;
use crossbeam::sync::SegQueue;

use nalgebra::coordinates::XYZW;

use ::utils::clamp;
use ::pixel::Pixel;
use ::mesh::{Mesh, Vertex};

use ::render::{FrameBuffer, ClipVertex, ScreenVertex, FaceWinding, Interpolate, Blend};

/// Defines the kinds of primitives that can be rendered by themselves.
///
/// When primitive sorting is used with the geometry shader stage, points are rendered first,
/// then lines, then triangles. This way their depth values, although equal to,
/// should be preserved compared to higher order primitives,
/// causing them to show up over them.
///
/// That is to say, points and lines should show up on top of triangles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Primitive {
    /// Individual points
    Point,
    /// Lines between two vertices
    Line,
    /// Triangles between three vertices
    Triangle,
    /// Exact same as `Triangle`,
    /// but renders a wireframe instead.
    Wireframe,
}

impl Primitive {
    #[inline]
    pub fn num_vertices(self) -> usize {
        match self {
            Primitive::Point => 1,
            Primitive::Line => 2,
            Primitive::Triangle | Primitive::Wireframe => 3,
        }
    }
}

// Internal type for accumulating varying primitives
#[derive(Clone)]
struct SeparablePrimitiveStorage<K> {
    points: Vec<ClipVertex<K>>,
    lines: Vec<ClipVertex<K>>,
    tris: Vec<ClipVertex<K>>,
}

impl<K> Default for SeparablePrimitiveStorage<K> {
    fn default() -> SeparablePrimitiveStorage<K> {
        SeparablePrimitiveStorage {
            points: Vec::new(),
            lines: Vec::new(),
            tris: Vec::new(),
        }
    }
}

// Internal type for accumulating varying primitives in screen-space
#[derive(Clone)]
struct SeparableScreenPrimitiveStorage<K> {
    points: Vec<ScreenVertex<K>>,
    lines: Vec<ScreenVertex<K>>,
    tris: Vec<ScreenVertex<K>>,
}

impl<K> Default for SeparableScreenPrimitiveStorage<K> {
    fn default() -> SeparableScreenPrimitiveStorage<K> {
        SeparableScreenPrimitiveStorage {
            points: Vec::new(),
            lines: Vec::new(),
            tris: Vec::new(),
        }
    }
}

impl<K> SeparablePrimitiveStorage<K> {
    /*
    fn get_primitives(&self, primitive: Primitive) -> &[ClipVertex<K>] {
        match primitive {
            Primitive::Point => &self.points,
            Primitive::Line => &self.lines,
            Primitive::Triangle => &self.tris,
        }
    }

    fn get_primitives_mut(&mut self, primitive: Primitive) -> &mut [ClipVertex<K>] {
        match primitive {
            Primitive::Point => &mut self.points,
            Primitive::Line => &mut self.lines,
            Primitive::Triangle => &mut self.tris,
        }
    }
    */

    fn append(&mut self, other: &mut SeparablePrimitiveStorage<K>) {
        self.points.append(&mut other.points);
        self.lines.append(&mut other.lines);
        self.tris.append(&mut other.tris);
    }

    #[inline]
    pub fn push_point(&mut self, point: ClipVertex<K>) {
        self.points.push(point);
    }

    #[inline]
    pub fn push_line(&mut self, start: ClipVertex<K>, end: ClipVertex<K>) {
        self.lines.reserve(2);
        self.lines.push(start);
        self.lines.push(end);
    }

    #[inline]
    pub fn push_triangle(&mut self, a: ClipVertex<K>, b: ClipVertex<K>, c: ClipVertex<K>) {
        self.tris.reserve(3);
        self.tris.push(a);
        self.tris.push(b);
        self.tris.push(c);
    }
}

/// Holds a reference to the internal storage structure for primitives
pub struct PrimitiveStorage<'s, K> where K: 's {
    storage: &'s mut SeparablePrimitiveStorage<K>,
}

impl<'s, K> PrimitiveStorage<'s, K> where K: 's {
    /// Adds a point to the storage
    #[inline(always)]
    pub fn emit_point(&mut self, point: ClipVertex<K>) {
        self.storage.push_point(point);
    }

    /// Adds a line to the storage
    #[inline(always)]
    pub fn emit_line(&mut self, start: ClipVertex<K>, end: ClipVertex<K>) {
        self.storage.push_line(start, end);
    }

    /// Adds a triangle to the storage
    #[inline(always)]
    pub fn emit_triangle(&mut self, a: ClipVertex<K>, b: ClipVertex<K>, c: ClipVertex<K>) {
        self.storage.push_triangle(a, b, c)
    }
}

/// Holds references to primitive vertices for each primitive type
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveRef<'p, K: 'p> {
    Point(&'p ClipVertex<K>),
    Line {
        start: &'p ClipVertex<K>,
        end: &'p ClipVertex<K>,
    },
    Triangle {
        a: &'p ClipVertex<K>,
        b: &'p ClipVertex<K>,
        c: &'p ClipVertex<K>,
    }
}

/// Holds mutable references to primitive vertices for each primitive type
#[derive(Debug)]
pub enum PrimitiveMut<'p, K: 'p> {
    Point(&'p mut ClipVertex<K>),
    Line {
        start: &'p mut ClipVertex<K>,
        end: &'p mut ClipVertex<K>,
    },
    Triangle {
        a: &'p mut ClipVertex<K>,
        b: &'p mut ClipVertex<K>,
        c: &'p mut ClipVertex<K>,
    }
}

// Internal type to simplify mesh-primitive representation
struct PrimitiveMesh<V> {
    mesh: Arc<Mesh<V>>,
    primitive: Primitive,
}

impl<V> Clone for PrimitiveMesh<V> {
    fn clone(&self) -> PrimitiveMesh<V> {
        PrimitiveMesh { mesh: self.mesh.clone(), primitive: self.primitive }
    }
}

impl<V> Deref for PrimitiveMesh<V> {
    type Target = Mesh<V>;

    #[inline(always)]
    fn deref(&self) -> &Mesh<V> { &*self.mesh }
}

/// Starting point for the rendering pipeline.
///
/// By itself, it only holds the framebuffer and global uniforms,
/// but it spawns the first shader stage using those.
pub struct Pipeline<U, P> where P: Pixel {
    framebuffer: FrameBuffer<P>,
    uniforms: U,
}

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
pub struct VertexShader<'a, V, U: 'a, P> where P: Pixel {
    framebuffer: &'a mut FrameBuffer<P>,
    uniforms: &'a U,
    mesh: PrimitiveMesh<V>,
}

/// Geometry shader stage
///
/// The geometry shader can edit and generate new vertices from the output of the vertex shader.
///
/// Examples of geometry shader usage are crude tessellation, vertex displacement,
/// and geometry visualisations like normal vector lines.
///
/// The geometry shader can be ran multiple times.
pub struct GeometryShader<'a, V, U: 'a, K, P> where P: Pixel {
    framebuffer: &'a mut FrameBuffer<P>,
    uniforms: &'a U,
    mesh: PrimitiveMesh<V>,
    indexed_vertices: Option<Vec<ClipVertex<K>>>,
    generated_primitives: SeparablePrimitiveStorage<K>,
}

/// Fragment shader stage.
///
/// The fragment shader is responsible for determining the color of pixels where the underlying geometry has been projected onto.
/// Usually this is individual triangles that are rasterized and shaded by the fragment shader, but it also supports point-cloud, wireframe,
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
pub struct FragmentShader<'a, V, U: 'a, K, P, B = ()> where P: Pixel {
    framebuffer: &'a mut FrameBuffer<P>,
    uniforms: &'a U,
    mesh: PrimitiveMesh<V>,
    indexed_vertices: Arc<Option<Vec<ScreenVertex<K>>>>,
    generated_primitives: Arc<SeparableScreenPrimitiveStorage<K>>,
    cull_faces: Option<FaceWinding>,
    blend: B,
}

///////////////////////

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
    pub fn render_mesh<V>(&mut self, primitive: Primitive, mesh: Arc<Mesh<V>>) -> VertexShader<V, U, P> where V: Send + Sync {
        VertexShader {
            mesh: PrimitiveMesh { mesh, primitive },
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

impl<'a, V, U: 'a, P> VertexShader<'a, V, U, P> where V: Send + Sync,
                                                      U: Send + Sync,
                                                      P: Pixel {
    /// Duplicates all references to internal state to return a cloned vertex shader,
    /// though since the vertex shader itself has very little internal state at this point,
    /// it's not that useful.
    pub fn duplicate<'b>(&'b mut self) -> VertexShader<'b, V, U, P> where 'a: 'b {
        VertexShader {
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh.clone(),
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
    pub fn run<S, K>(self, vertex_shader: S) -> GeometryShader<'a, V, U, K, P> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                     K: Send + Sync + Interpolate {
        let VertexShader {
            mesh,
            uniforms,
            framebuffer
        } = self;

        let indexed_vertices = mesh.vertices.par_iter()
                                            .map(|vertex| vertex_shader(vertex, uniforms))
                                            .collect();

        GeometryShader {
            mesh: mesh,
            uniforms: uniforms,
            framebuffer: framebuffer,
            indexed_vertices: Some(indexed_vertices),
            generated_primitives: SeparablePrimitiveStorage::default(),
        }
    }

    /// Same as `run`, but skips the geometry shader stage.
    ///
    /// This pathway does not do any clipping, so beware of that when rendering. However,
    /// it is the fastest path, so the tradeoff may be acceptable for some use cases.
    pub fn run_to_fragment<S, K>(self, vertex_shader: S) -> FragmentShader<'a, V, U, K, P, ()> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                                     K: Send + Sync + Interpolate {
        let VertexShader {
            mesh,
            uniforms,
            framebuffer
        } = self;

        let viewport = framebuffer.viewport();

        let indexed_vertices = mesh.vertices.par_iter()
                                            .map(|vertex| vertex_shader(vertex, uniforms)
                                                .normalize(viewport))
                                            .collect();

        FragmentShader {
            framebuffer: framebuffer,
            uniforms: uniforms,
            mesh: mesh,
            indexed_vertices: Arc::new(Some(indexed_vertices)),
            generated_primitives: Arc::new(SeparableScreenPrimitiveStorage::default()),
            cull_faces: None,
            blend: (),
        }
    }
}

impl<'a, V, U: 'a, K, P> GeometryShader<'a, V, U, K, P> where V: Send + Sync,
                                                              U: Send + Sync,
                                                              K: Send + Sync,
                                                              P: Pixel {
    pub fn finish(self) -> FragmentShader<'a, V, U, K, P, ()> {
        let viewport = self.framebuffer.viewport();

        let SeparablePrimitiveStorage { points, lines, tris } = self.generated_primitives;

        let normalize_vertex = move |vertex: ClipVertex<K>| { vertex.normalize(viewport) };

        let indexed_vertices = self.indexed_vertices.map(|indexed_vertices| {
            indexed_vertices.into_par_iter().map(&normalize_vertex)
        });

        let points = points.into_par_iter().map(&normalize_vertex);
        let lines = lines.into_par_iter().map(&normalize_vertex);
        let tris = tris.into_par_iter().map(&normalize_vertex);

        FragmentShader {
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh,
            indexed_vertices: Arc::new(indexed_vertices.map(|mapped_vertices| {
                mapped_vertices.collect()
            })),
            generated_primitives: Arc::new(SeparableScreenPrimitiveStorage {
                points: points.collect(),
                lines: lines.collect(),
                tris: tris.collect(),
            }),
            cull_faces: None,
            blend: (),
        }
    }
}

impl<'a, V, U: 'a, K, P> GeometryShader<'a, V, U, K, P> where V: Send + Sync,
                                                              U: Send + Sync,
                                                              K: Send + Sync,
                                                              P: Pixel {
    pub fn duplicate<'b>(&'b mut self) -> GeometryShader<'b, V, U, K, P> where 'a: 'b, K: Clone {
        /// Duplicate the geometry shader, and copies any processed geometry.
        ///
        /// Geometry are not synced between duplicated geometry shaders.
        GeometryShader {
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh.clone(),
            indexed_vertices: self.indexed_vertices.clone(),
            generated_primitives: self.generated_primitives.clone(),
        }
    }

    /// Runs the geometry shader, replacing all primitives with the generated primitives.
    ///
    /// To run the geometry shader in-place to modify/append primitives, use `modify` or `append`
    pub fn replace<S>(self, geometry_shader: S) -> Self
        where S: for<'s, 'p> Fn(PrimitiveStorage<'s, K>, PrimitiveRef<'p, K>, &U) + Send + Sync + 'static {
        let GeometryShader { mesh, framebuffer, uniforms, indexed_vertices, generated_primitives } = self;

        let from_points = generated_primitives.points
            .par_chunks(1).with_min_len(generated_primitives.points.len() / (1 * current_num_threads()))
            .fold(|| SeparablePrimitiveStorage::default(), |mut storage, point| {
                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                PrimitiveRef::Point(&point[0]),
                                uniforms);

                storage
            });

        let from_lines = generated_primitives.lines
            .par_chunks(2).with_min_len(generated_primitives.lines.len() / (2 * current_num_threads()))
            .fold(|| SeparablePrimitiveStorage::default(), |mut storage, line| {
                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                PrimitiveRef::Line { start: &line[0], end: &line[1] },
                                uniforms);

                storage
            });

        let from_tris = generated_primitives.tris
            .par_chunks(3).with_min_len(generated_primitives.tris.len() / (3 * current_num_threads()))
            .fold(|| SeparablePrimitiveStorage::default(), |mut storage, triangle| {
                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                PrimitiveRef::Triangle { a: &triangle[0], b: &triangle[1], c: &triangle[2] },
                                uniforms);

                storage
            });

        let new_primitives = {
            if let Some(indexed_vertices) = indexed_vertices {
                let num_vertices = mesh.primitive.num_vertices();

                let primitives_per_thread = mesh.indices.len() / (num_vertices * current_num_threads());

                let from_indexed = mesh.indices
                    .par_chunks(num_vertices)
                    .with_min_len(primitives_per_thread)
                    .fold(|| SeparablePrimitiveStorage::default(), {
                        let fold_method: Box<Fn(SeparablePrimitiveStorage<K>, &[u32]) -> SeparablePrimitiveStorage<K> + Sync> = match mesh.primitive {
                            Primitive::Point => Box::new(|mut storage, primitive| {
                                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                                PrimitiveRef::Point(&indexed_vertices[primitive[0] as usize]),
                                                uniforms);
                                storage
                            }),
                            Primitive::Line => Box::new(|mut storage, primitive| {
                                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                                PrimitiveRef::Line {
                                                    start: &indexed_vertices[primitive[0] as usize],
                                                    end: &indexed_vertices[primitive[1] as usize],
                                                },
                                                uniforms);
                                storage
                            }),
                            Primitive::Triangle | Primitive::Wireframe => Box::new(|mut storage, primitive| {
                                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                                PrimitiveRef::Triangle {
                                                    a: &indexed_vertices[primitive[0] as usize],
                                                    b: &indexed_vertices[primitive[1] as usize],
                                                    c: &indexed_vertices[primitive[2] as usize],
                                                },
                                                uniforms);
                                storage
                            })
                        };

                        move |storage, primitive| { fold_method(storage, primitive) }
                    });

                from_indexed.chain(from_points).chain(from_lines).chain(from_tris).reduce_with(|mut storage_a, mut storage_b| {
                    storage_a.append(&mut storage_b);
                    storage_a
                })
            } else {
                from_points.chain(from_lines).chain(from_tris).reduce_with(|mut storage_a, mut storage_b| {
                    storage_a.append(&mut storage_b);
                    storage_a
                })
            }
        };

        let mut replaced_primitives = SeparablePrimitiveStorage::default();

        if let Some(mut new_primitives) = new_primitives {
            replaced_primitives.append(&mut new_primitives);
        };

        GeometryShader {
            mesh,
            framebuffer,
            uniforms,
            indexed_vertices: None,
            generated_primitives: replaced_primitives,
        }
    }

    /// Runs the geometry shader, modifying existing primitives and appending new ones.
    ///
    /// For when not replacing all existing primitives with new ones, this will be more efficient.
    pub fn modify<S>(self, geometry_shader: S) -> Self
        where S: for<'s, 'p> Fn(PrimitiveStorage<'s, K>, PrimitiveMut<'p, K>, &U) + Send + Sync + 'static, K: Clone {
        let GeometryShader { mesh, framebuffer, uniforms, indexed_vertices, mut generated_primitives } = self;

        let new_primitives = {
            let points_per_thread = generated_primitives.points.len() / (1 * current_num_threads());
            let lines_per_thread = generated_primitives.lines.len() / (2 * current_num_threads());
            let triangles_per_thread = generated_primitives.tris.len() / (3 * current_num_threads());

            let from_points = generated_primitives.points
                .par_chunks_mut(1).with_min_len(points_per_thread)
                .fold(|| SeparablePrimitiveStorage::default(), |mut storage, point| {
                    geometry_shader(PrimitiveStorage { storage: &mut storage },
                                    PrimitiveMut::Point(&mut point[0]),
                                    uniforms);

                    storage
                });

            let from_lines = generated_primitives.lines
                .par_chunks_mut(2).with_min_len(lines_per_thread)
                .fold(|| SeparablePrimitiveStorage::default(), |mut storage, line| {
                    let (mut start, mut end) = line.split_at_mut(1);

                    geometry_shader(PrimitiveStorage { storage: &mut storage },
                                    PrimitiveMut::Line { start: &mut start[0], end: &mut end[0] },
                                    uniforms);

                    storage
                });

            let from_tris = generated_primitives.tris
                .par_chunks_mut(3).with_min_len(triangles_per_thread)
                .fold(|| SeparablePrimitiveStorage::default(), |mut storage, triangle| {
                    let (mut a, mut bc) = triangle.split_at_mut(1);
                    let (mut b, mut c) = bc.split_at_mut(1);

                    geometry_shader(PrimitiveStorage { storage: &mut storage },
                                    PrimitiveMut::Triangle { a: &mut a[0], b: &mut b[0], c: &mut c[0] },
                                    uniforms);

                    storage
                });

            if let Some(indexed_vertices) = indexed_vertices {
                let num_vertices = mesh.primitive.num_vertices();

                let primitives_per_thread = mesh.indices.len() / (num_vertices * current_num_threads());

                let from_indexed = mesh.indices
                    .par_chunks(num_vertices)
                    .with_min_len(primitives_per_thread)
                    .fold(|| SeparablePrimitiveStorage::default(), {
                        // For modify, all indexed primitives must be cloned before modification,
                        // then inserted into the generated primitives collection. This way we can group
                        // together the de-indexing and geometry shader modifications.
                        let fold_method: Box<Fn(SeparablePrimitiveStorage<K>, &[u32]) -> SeparablePrimitiveStorage<K> + Sync> = match mesh.primitive {
                            Primitive::Point => Box::new(|mut storage, primitive| {
                                let mut point = indexed_vertices[primitive[0] as usize].clone();

                                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                                PrimitiveMut::Point(&mut point),
                                                uniforms);

                                storage.push_point(point);

                                storage
                            }),
                            Primitive::Line => Box::new(|mut storage, primitive| {
                                let mut start = indexed_vertices[primitive[0] as usize].clone();
                                let mut end = indexed_vertices[primitive[1] as usize].clone();

                                geometry_shader(PrimitiveStorage { storage: &mut storage }, PrimitiveMut::Line {
                                    start: &mut start,
                                    end: &mut end,
                                }, uniforms);

                                storage.push_line(start, end);

                                storage
                            }),
                            Primitive::Triangle | Primitive::Wireframe => Box::new(|mut storage, primitive| {
                                let mut a = indexed_vertices[primitive[0] as usize].clone();
                                let mut b = indexed_vertices[primitive[1] as usize].clone();
                                let mut c = indexed_vertices[primitive[2] as usize].clone();

                                geometry_shader(PrimitiveStorage { storage: &mut storage }, PrimitiveMut::Triangle {
                                    a: &mut a,
                                    b: &mut b,
                                    c: &mut c,
                                }, uniforms);

                                storage.push_triangle(a, b, c);

                                storage
                            }),
                        };

                        move |storage, primitive| { fold_method(storage, primitive) }
                    });

                from_indexed.chain(from_points).chain(from_lines).chain(from_tris).reduce_with(|mut storage_a, mut storage_b| {
                    storage_a.append(&mut storage_b);
                    storage_a
                })
            } else {
                from_points.chain(from_lines).chain(from_tris).reduce_with(|mut storage_a, mut storage_b| {
                    storage_a.append(&mut storage_b);
                    storage_a
                })
            }
        };

        if let Some(mut new_primitives) = new_primitives {
            generated_primitives.append(&mut new_primitives);
        }

        GeometryShader {
            mesh,
            framebuffer,
            uniforms,
            indexed_vertices: None,
            generated_primitives,
        }
    }

    /// An append-only geometry shader that is more efficient than `replace` or `modify` variations.
    pub fn append<S>(self, geometry_shader: S) -> Self
        where S: for<'s, 'p> Fn(PrimitiveStorage<'s, K>, PrimitiveRef<'p, K>, &U) + Send + Sync + 'static {
        let GeometryShader { mesh, framebuffer, uniforms, indexed_vertices, mut generated_primitives } = self;

        let new_primitives = {
            let from_points = generated_primitives.points
                .par_chunks(1).with_min_len(generated_primitives.points.len() / (1 * current_num_threads()))
                .fold(|| SeparablePrimitiveStorage::default(), |mut storage, point| {
                    geometry_shader(PrimitiveStorage { storage: &mut storage },
                                    PrimitiveRef::Point(&point[0]),
                                    uniforms);

                    storage
                });

            let from_lines = generated_primitives.lines
                .par_chunks(2).with_min_len(generated_primitives.lines.len() / (2 * current_num_threads()))
                .fold(|| SeparablePrimitiveStorage::default(), |mut storage, line| {
                    geometry_shader(PrimitiveStorage { storage: &mut storage },
                                    PrimitiveRef::Line { start: &line[0], end: &line[1] },
                                    uniforms);

                    storage
                });

            let from_tris = generated_primitives.tris
                .par_chunks(3).with_min_len(generated_primitives.tris.len() / (3 * current_num_threads()))
                .fold(|| SeparablePrimitiveStorage::default(), |mut storage, triangle| {
                    geometry_shader(PrimitiveStorage { storage: &mut storage },
                                    PrimitiveRef::Triangle { a: &triangle[0], b: &triangle[1], c: &triangle[2] },
                                    uniforms);

                    storage
                });

            if let Some(ref indexed_vertices) = indexed_vertices {
                let num_vertices = mesh.primitive.num_vertices();

                let primitives_per_thread = mesh.indices.len() / (num_vertices * current_num_threads());

                let from_indexed = mesh.indices
                    .par_chunks(num_vertices)
                    .with_min_len(primitives_per_thread)
                    .fold(|| SeparablePrimitiveStorage::default(), {
                        let fold_method: Box<Fn(SeparablePrimitiveStorage<K>, &[u32]) -> SeparablePrimitiveStorage<K> + Sync> = match mesh.primitive {
                            Primitive::Point => Box::new(|mut storage, primitive| {
                                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                                PrimitiveRef::Point(&indexed_vertices[primitive[0] as usize]),
                                                uniforms);
                                storage
                            }),
                            Primitive::Line => Box::new(|mut storage, primitive| {
                                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                                PrimitiveRef::Line {
                                                    start: &indexed_vertices[primitive[0] as usize],
                                                    end: &indexed_vertices[primitive[1] as usize],
                                                },
                                                uniforms);
                                storage
                            }),
                            Primitive::Triangle | Primitive::Wireframe => Box::new(|mut storage, primitive| {
                                geometry_shader(PrimitiveStorage { storage: &mut storage },
                                                PrimitiveRef::Triangle {
                                                    a: &indexed_vertices[primitive[0] as usize],
                                                    b: &indexed_vertices[primitive[1] as usize],
                                                    c: &indexed_vertices[primitive[2] as usize],
                                                },
                                                uniforms);
                                storage
                            })
                        };

                        move |storage, primitive| { fold_method(storage, primitive) }
                    });

                from_indexed.chain(from_points).chain(from_lines).chain(from_tris).reduce_with(|mut storage_a, mut storage_b| {
                    storage_a.append(&mut storage_b);
                    storage_a
                })
            } else {
                from_points.chain(from_lines).chain(from_tris).reduce_with(|mut storage_a, mut storage_b| {
                    storage_a.append(&mut storage_b);
                    storage_a
                })
            }
        };

        if let Some(mut new_primitives) = new_primitives {
            generated_primitives.append(&mut new_primitives);
        }

        GeometryShader {
            mesh,
            framebuffer,
            uniforms,
            indexed_vertices,
            generated_primitives,
        }
    }

    pub fn clip_primitives(self) -> Self where K: Interpolate {
        unimplemented!()
    }
}

/// Fragment returned by the fragment shader, which can either be a color
/// value for the pixel or a discard flag to skip that fragment altogether.
#[derive(Debug, Clone, Copy)]
pub enum Fragment<P> where P: Pixel {
    /// Discard the fragment altogether, as if it was never there.
    Discard,
    /// Desired color for the pixel
    Color(P)
}

impl<'a, V, U: 'a, K, P, B> Deref for FragmentShader<'a, V, U, K, P, B> where P: Pixel,
                                                                              B: Blend<P> {
    type Target = B;
    fn deref(&self) -> &B { &self.blend }
}

impl<'a, V, U: 'a, K, P, B> DerefMut for FragmentShader<'a, V, U, K, P, B> where P: Pixel,
                                                                                 B: Blend<P> {
    fn deref_mut(&mut self) -> &mut B { &mut self.blend }
}

impl<'a, V, U: 'a, K, P, O> FragmentShader<'a, V, U, K, P, O> where P: Pixel {
    pub fn with_blend<B>(self, blend: B) -> FragmentShader<'a, V, U, K, P, B> where B: Blend<P> {
        FragmentShader {
            blend: blend,
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh,
            indexed_vertices: self.indexed_vertices,
            generated_primitives: self.generated_primitives,
            cull_faces: self.cull_faces,
        }
    }

    pub fn with_default_blend<B>(self) -> FragmentShader<'a, V, U, K, P, B> where B: Blend<P> + Default {
        self.with_blend(B::default())
    }
}

impl<'a, V, U: 'a, K, P, B> FragmentShader<'a, V, U, K, P, B> where V: Send + Sync,
                                                                    U: Send + Sync,
                                                                    K: Send + Sync + Interpolate,
                                                                    P: Pixel, B: Blend<P> {
    /// Duplicates all references to internal state to return a cloned fragment shader,
    /// which can be used to efficiently render the same geometry with different
    /// rasterization methods in quick succession.
    pub fn duplicate<'b>(&'b mut self) -> FragmentShader<'b, V, U, K, P, B> where 'a: 'b,
                                                                                  B: Clone {
        FragmentShader {
            framebuffer: self.framebuffer,
            uniforms: self.uniforms,
            mesh: self.mesh.clone(),
            indexed_vertices: self.indexed_vertices.clone(),
            generated_primitives: self.generated_primitives.clone(),
            cull_faces: self.cull_faces.clone(),
            blend: self.blend.clone(),
        }
    }

    /// Cull faces based on winding order. For more information on how and why this works,
    /// check out the documentation for the [`FaceWinding`](../geometry/enum.FaceWinding.html) enum.
    #[inline(always)]
    pub fn cull_faces(&mut self, cull: Option<FaceWinding>) {
        self.cull_faces = cull;
    }

    pub fn run<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        let FragmentShader {
            mesh,
            uniforms,
            framebuffer,
            indexed_vertices,
            generated_primitives,
            cull_faces,
            blend
        } = self;

        let (width, height) = (framebuffer.width() as usize,
                               framebuffer.height() as usize);

        let bb = ((width - 1) as f32,
                  (height - 1) as f32);

        let plot_point = |framebuffer: &mut FrameBuffer<P>,
                          point: &ScreenVertex<K>| {
            let XYZW { x, y, z, .. } = *point.position;

            if 0.0 <= x && x < bb.0 && 0.0 <= y && y < bb.1 && z > 0.0 {
                let px = x as u32;
                let py = y as u32;

                let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(px, py) };

                if z < *fd {
                    match fragment_shader(point, &uniforms) {
                        Fragment::Color(c) => {
                            *fc = blend.blend(c, *fc);
                            *fd = z;
                        }
                        Fragment::Discard => ()
                    };
                }
            }
        };

        let draw_line = |framebuffer: &mut FrameBuffer<P>,
                         start: &ScreenVertex<K>,
                         end: &ScreenVertex<K>| {
            let XYZW { x: x1, y: y1, .. } = *start.position;
            let XYZW { x: x2, y: y2, .. } = *end.position;

            let d = (x1 - x2).hypot(y1 - y2);

            let plot_fragment = |x, y, alpha: f64| {
                if x >= 0 && y >= 0 {
                    let xf = x as f32;
                    let yf = y as f32;

                    let x = x as u32;
                    let y = y as u32;

                    if 0.0 <= xf && xf < bb.0 && 0.0 <= yf && yf < bb.1 {
                        let d1 = (x1 - xf).hypot(y1 - yf);

                        let t = d1 / d;

                        let position = Interpolate::linear_interpolate(t, &start.position, &end.position);

                        let z = position.z;

                        if z > 0.0 {
                            let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(x, y) };

                            if z < *fd {
                                let fragment = fragment_shader(&ScreenVertex {
                                    position,
                                    uniforms: Interpolate::linear_interpolate(t, &start.uniforms, &end.uniforms)
                                }, &uniforms);

                                match fragment {
                                    Fragment::Color(c) => {
                                        *fc = blend.blend(c.mul_alpha(alpha as f32), *fc);
                                        *fd = z;
                                    }
                                    Fragment::Discard => ()
                                }
                            }
                        }
                    }
                }
            };

            ::render::line::draw_line_xiaolin_wu(x1 as f64, y1 as f64, x2 as f64, y2 as f64, plot_fragment);
        };

        let rasterize_triangle = |framebuffer: &mut FrameBuffer<P>,
                                  a: &ScreenVertex<K>,
                                  b: &ScreenVertex<K>,
                                  c: &ScreenVertex<K>| {
            let XYZW { x: x1, y: y1, .. } = *a.position;
            let XYZW { x: x2, y: y2, .. } = *b.position;
            let XYZW { x: x3, y: y3, .. } = *c.position;

            // do backface culling
            if let Some(winding) = cull_faces {
                let a = x1 * y2 + x2 * y3 + x3 * y1 - x2 * y1 - x3 * y2 - x1 * y3;

                if winding == if a.is_sign_negative() { FaceWinding::Clockwise } else { FaceWinding::CounterClockwise } {
                    return;
                }
            }

            // calculate determinant
            let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);

            // find x bounds for the bounding box
            let min_x: usize = clamp(x1.min(x2).min(x3), 0.0, bb.0) as usize;
            let max_x: usize = clamp(x1.max(x2).max(x3), 0.0, bb.0) as usize;

            // find y bounds for the bounding box
            let min_y: usize = clamp(y1.min(y2).min(y3), 0.0, bb.1) as usize;
            let max_y: usize = clamp(y1.max(y2).max(y3), 0.0, bb.1) as usize;

            let dx = width - (max_x - min_x + 1);

            let (color, depth) = framebuffer.buffers_mut();

            let mut index = min_y * width + min_x;

            let mut py = min_y;

            while py <= max_y {
                let mut px = min_x;

                while px <= max_x {
                    // Real screen position should be in the center of the pixel.
                    let (x, y) = (px as f32 + 0.5,
                                  py as f32 + 0.5);

                    // calculate barycentric coordinates of the current point
                    let u = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
                    let v = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
                    let w = 1.0 - u - v;

                    // check if the point is inside the triangle at all
                    if !(u < 0.0 || v < 0.0 || w < 0.0) {
                        // interpolate screen-space position
                        let position = a.position * u + b.position * v + c.position * w;

                        let z = position.z;

                        // don't render pixels "behind" the camera
                        if z > 0.0 {
                            let fd = unsafe { depth.get_unchecked_mut(index) };

                            // skip fragments that are behind over previous fragments
                            if z < *fd || blend.ignore_depth() {
                                // run fragment shader
                                let fragment = fragment_shader(&ScreenVertex {
                                    position: position,
                                    // interpolate the uniforms
                                    uniforms: Interpolate::barycentric_interpolate(u, &a.uniforms,
                                                                                   v, &b.uniforms,
                                                                                   w, &c.uniforms),
                                }, &*uniforms);

                                match fragment {
                                    Fragment::Color(c) => {
                                        let fc = unsafe { color.get_unchecked_mut(index) };

                                        *fc = blend.blend(c, *fc);
                                        *fd = z;
                                    }
                                    Fragment::Discard => ()
                                };
                            }
                        }
                    }

                    px += 1;
                    index += 1;
                }

                py += 1;
                index += dx;
            }
        };

        let framebuffer = Mutex::new(framebuffer);

        if let Some(ref indexed_vertices) = *indexed_vertices {
            let num_vertices = mesh.primitive.num_vertices();

            let primitives_per_thread = mesh.indices.len() / (num_vertices * current_num_threads());

            let partial_framebuffers = mesh.indices.par_chunks(num_vertices).with_min_len(primitives_per_thread).fold(
                || framebuffer.lock().unwrap().empty_clone(), {
                    let fold_method: Box<Fn(FrameBuffer<P>, &[u32]) -> FrameBuffer<P> + Sync> = match mesh.primitive {
                        Primitive::Point => Box::new(|mut framebuffer, primitive| {
                            plot_point(&mut framebuffer, &indexed_vertices[primitive[0] as usize]);

                            framebuffer
                        }),
                        Primitive::Line => Box::new(|mut framebuffer, primitive| {
                            draw_line(&mut framebuffer,
                                      &indexed_vertices[primitive[0] as usize],
                                      &indexed_vertices[primitive[1] as usize]);

                            framebuffer
                        }),
                        Primitive::Triangle => Box::new(|mut framebuffer, primitive| {
                            rasterize_triangle(&mut framebuffer,
                                               &indexed_vertices[primitive[0] as usize],
                                               &indexed_vertices[primitive[1] as usize],
                                               &indexed_vertices[primitive[2] as usize],
                            );

                            framebuffer
                        }),
                        Primitive::Wireframe => Box::new(|mut framebuffer, primitive| {
                            draw_line(&mut framebuffer, &indexed_vertices[primitive[0] as usize], &indexed_vertices[primitive[1] as usize]);
                            draw_line(&mut framebuffer, &indexed_vertices[primitive[1] as usize], &indexed_vertices[primitive[2] as usize]);
                            draw_line(&mut framebuffer, &indexed_vertices[primitive[2] as usize], &indexed_vertices[primitive[0] as usize]);

                            framebuffer
                        }),
                    };

                    move |framebuffer, primitive| { fold_method(framebuffer, primitive) }
                });

            partial_framebuffers.reduce_with(|mut a, mut b| {
                b.merge_into(&mut a, &blend);
                framebuffer.lock().unwrap().cache_empty_clone(b);
                a
            }).map(|mut final_framebuffer| {
                let mut framebuffer = framebuffer.lock().unwrap();
                // Merge final framebuffer into external framebuffer
                final_framebuffer.merge_into(&mut *framebuffer, &blend);
                framebuffer.cache_empty_clone(final_framebuffer)
            });
        }
    }

    /*
        pub fn run<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        let framebuffer = Mutex::new(framebuffer);

        let partial_framebuffers_from_indexed = if let Some(indexed_vertices) = indexed_vertices {
            let index_queue: SegQueue<&[u32]> = SegQueue::new();

            // Use chunks of 1024 triangles, giving a balance between granularity and per-chunk performance.
            // For example, a mesh with 4 million triangles will have about 4,000 chunks, and a mesh with 16,000 triangles will have
            // about 16 chunks, so even on small meshes there is a chance for threads to steal the other's work just a little.
            for chunk in mesh.indices.chunks(3 * 1024) {
                index_queue.push(chunk);
            }

            (0..current_num_threads()).into_par_iter().map(|_| -> FrameBuffer<P> {
                let mut framebuffer = framebuffer.lock().unwrap().empty_clone();

                while let Some(chunk) = index_queue.try_pop() {
                    for triangle in chunk.chunks(3) {
                        // skip incomplete triangles
                        if triangle.len() != 3 { continue; }

                        let ref a = indexed_vertices[triangle[0] as usize];
                        let ref b = indexed_vertices[triangle[1] as usize];
                        let ref c = indexed_vertices[triangle[2] as usize];

                        rasterize_triangle(&mut framebuffer, a, b, c);
                    }
                }

                framebuffer
            })
        } else {
            None.into_par_iter()
        };

        // Only allow as many new empty framebuffer clones as their are running threads, so one framebuffer per thread.
        // This has the benefit of running a large of number of triangles sequentially.
        let triangles_per_thread = mesh.indices.len() / (3 * current_num_threads());

        let partial_framebuffers_from_created = created_vertices.par_chunks(3).with_min_len(triangles_per_thread).fold(
            || { framebuffer.lock().unwrap().empty_clone() }, |mut framebuffer, triangle| {
                if triangle.len() == 3 {
                    rasterize_triangle(&mut framebuffer, &triangle[0], &triangle[1], &triangle[2]);
                }

                framebuffer
            });

        // Merge incoming partial framebuffers in parallel
        partial_framebuffers.reduce_with(|mut a, mut b| {
            b.merge_into(&mut a, &blend);
            framebuffer.lock().unwrap().cache_empty_clone(b);
            a
        }).map(|mut final_framebuffer| {
            let mut framebuffer = framebuffer.lock().unwrap();
            // Merge final framebuffer into external framebuffer
            final_framebuffer.merge_into(&mut *framebuffer, &blend);
            framebuffer.cache_empty_clone(final_framebuffer)
        });
    }
    */
}