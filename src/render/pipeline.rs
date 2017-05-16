//! Rendering pipeline implementation

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::marker::PhantomData;

use rayon;
use rayon::prelude::*;

use nalgebra::coordinates::XYZW;

use ::utils::clamp;
use ::pixel::Pixel;
use ::mesh::{Mesh, Vertex};

use ::render::{FrameBuffer, ClipVertex, ScreenVertex, FaceWinding, Interpolate, Blend};
use ::render::{Primitive, PrimitiveRef};
use ::render::primitive::{Point, Line, Triangle};
use ::render::clip::ALL_CLIPPING_PLANES;

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
    inner: &'s mut SeparablePrimitiveStorage<K>,
}

impl<'s, K> PrimitiveStorage<'s, K> where K: 's {
    /// Adds a point to the storage
    #[inline(always)]
    pub fn emit_point(&mut self, point: ClipVertex<K>) {
        self.inner.push_point(point);
    }

    /// Adds a line to the storage
    #[inline(always)]
    pub fn emit_line(&mut self, start: ClipVertex<K>, end: ClipVertex<K>) {
        self.inner.push_line(start, end);
    }

    /// Adds a triangle to the storage
    #[inline(always)]
    pub fn emit_triangle(&mut self, a: ClipVertex<K>, b: ClipVertex<K>, c: ClipVertex<K>) {
        self.inner.push_triangle(a, b, c)
    }

    pub fn re_emit<'p>(&mut self, primitive: PrimitiveRef<'p, K>) where K: Clone {
        match primitive {
            PrimitiveRef::Point(point) => self.emit_point(point.clone()),
            PrimitiveRef::Line { start, end } => self.emit_line(start.clone(), end.clone()),
            PrimitiveRef::Triangle { a, b, c } => self.emit_triangle(a.clone(), b.clone(), c.clone()),
        }
    }
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
pub struct VertexShader<'a, T, V, U: 'a, P> where P: Pixel, T: Primitive {
    framebuffer: &'a mut FrameBuffer<P>,
    uniforms: &'a U,
    mesh: Arc<Mesh<V>>,
    indexed_primitive: PhantomData<T>,
}

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

pub const DEFAULT_TILE_SIZE: (f32, f32) = (256, 256);

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

impl<'a, T, V, U: 'a, P> VertexShader<'a, T, V, U, P> where T: Primitive,
                                                            V: Send + Sync,
                                                            U: Send + Sync,
                                                            P: Pixel {
    /// Duplicates all references to internal state to return a cloned vertex shader,
    /// though since the vertex shader itself has very little internal state at this point,
    /// it's not that useful.
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
    pub fn run_to_fragment<S, K>(self, vertex_shader: S) -> FragmentShader<'a, T, V, U, K, P, ()> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                                        K: Send + Sync + Interpolate {
        let VertexShader {
            framebuffer,
            uniforms,
            mesh,
            ..
        } = self;

        let viewport = framebuffer.viewport();

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

impl<'a, T, V, U: 'a, K, P> GeometryShader<'a, T, V, U, K, P> where T: Primitive,
                                                                    V: Send + Sync,
                                                                    U: Send + Sync,
                                                                    K: Send + Sync,
                                                                    P: Pixel {
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

    pub fn finish(self) -> FragmentShader<'a, T, V, U, K, P, ()> {
        let GeometryShader { framebuffer, uniforms, mesh, indexed_primitive, indexed_vertices, generated_primitives } = self;

        let viewport = framebuffer.viewport();

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

    pub fn run<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        let FragmentShader {
            mesh,
            uniforms,
            framebuffer,
            indexed_vertices,
            generated_primitives,
            cull_faces,
            blend,
            antialiased_lines,
            tile_size,
            ..
        } = self;

        let (width, height) = (framebuffer.width() as usize,
                               framebuffer.height() as usize);

        let bb = (width as f32, height as f32);
    }
}