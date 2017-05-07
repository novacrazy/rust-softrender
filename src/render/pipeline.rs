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

use ::render::blend::Blend;
use ::render::geometry::{FaceWinding, ClipVertex, ScreenVertex};
use ::render::framebuffer::FrameBuffer;
use ::render::uniform::Barycentric;

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
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
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
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
    indexed_vertices: Vec<ClipVertex<K>>,
    created_vertices: Vec<ClipVertex<K>>,
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
/// Uniforms passed from the vertex shader are interpolating inside the triangles using Barycentric interpolation,
/// which is why it must satisfy the [`Barycentric`](../uniform/trait.Barycentric.html) trait, which can be automatically implemented for many types using the
/// `declare_uniforms!` macro. See the documentation on that for more information on how to use it.
pub struct FragmentShader<'a, V, U: 'a, K, P, B = ()> where P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
    indexed_vertices: Arc<Vec<ScreenVertex<K>>>,
    created_vertices: Arc<Vec<ScreenVertex<K>>>,
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
    pub fn render_mesh<V>(&mut self, mesh: Arc<Mesh<V>>) -> VertexShader<V, U, P> where V: Send + Sync {
        VertexShader {
            mesh: mesh,
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
            mesh: self.mesh.clone(),
            uniforms: self.uniforms,
            framebuffer: self.framebuffer
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
                                                                                     K: Send + Sync + Barycentric {
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
            indexed_vertices: indexed_vertices,
            created_vertices: Vec::new(),
        }
    }

    /// Same as `run`, but skips the geometry shader stage.
    ///
    /// This pathway does not do any clipping, so beware of that when rendering. However,
    /// it is the fastest path, so the tradeoff may be acceptable for some use cases.
    pub fn run_to_fragment<S, K, B>(self, vertex_shader: S) -> FragmentShader<'a, V, U, K, P, ()> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                                        K: Send + Sync + Barycentric {
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
            mesh: mesh,
            uniforms: uniforms,
            framebuffer: framebuffer,
            indexed_vertices: Arc::new(indexed_vertices),
            created_vertices: Arc::new(Vec::new()),
            cull_faces: None,
            blend: (),
        }
    }
}

impl<'a, V, U: 'a, K, P> GeometryShader<'a, V, U, K, P> where V: Send + Sync,
                                                              U: Send + Sync,
                                                              K: Send + Sync + Barycentric,
                                                              P: Pixel {
    pub fn duplicate<'b>(&'b mut self) -> GeometryShader<'b, V, U, K, P> where 'a: 'b, K: Clone {
        /// Duplicate the geometry shader, and copies any processed geometry.
        ///
        /// Geometry are not synced between duplicated geometry shaders.
        GeometryShader {
            mesh: self.mesh.clone(),
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            indexed_vertices: self.indexed_vertices.clone(),
            created_vertices: self.created_vertices.clone()
        }
    }

    /// Runs the geometry shader on triangle primitives.
    ///
    /// See the documentation for `run_generic` for more info on how it works.
    #[inline]
    pub fn triangles<S>(self, geometry_shader: S) -> GeometryShader<'a, V, U, K, P> where S: Fn(&mut [ClipVertex<K>], &U) -> Option<Vec<ClipVertex<K>>> + Send + Sync + 'static {
        self.run_generic(geometry_shader, 3)
    }

    /// Runs the geometry shader with the given number of vertices per primitive. For example, a triangle primitive would be three vertices.
    ///
    /// The geometry shader is allowed to modify existing vertices outputted by the vertex shader via it's parameters,
    /// but also generate entire new primitives by returning them in a `Vec`.
    ///
    /// If the number of vertices returned by the geometry shader is not a multiple of the number of vertices,
    /// the result is discarded, so make sure it's correct.
    pub fn run_generic<S>(self, geometry_shader: S, primitive_vertices: usize) -> GeometryShader<'a, V, U, K, P> where S: Fn(&mut [ClipVertex<K>], &U) -> Option<Vec<ClipVertex<K>>> + Send + Sync + 'static {
        let GeometryShader {
            mesh,
            uniforms,
            framebuffer,
            mut indexed_vertices,
            mut created_vertices,
        } = self;

        let new_vertices_grouped: Vec<Vec<ClipVertex<K>>> = indexed_vertices.par_chunks_mut(primitive_vertices).filter_map(|primitive| {
            if primitive.len() == primitive_vertices {
                geometry_shader(primitive, uniforms).and_then(|new_vertices| {
                    // Only accept new primitives of the same length
                    if new_vertices.len() % primitive_vertices == 0 { Some(new_vertices) } else { None }
                })
            } else { None }
        }).collect();

        // Run through the new vertices really quick and accumulate their total length
        let total_new_vertices = new_vertices_grouped.iter().fold(0, |len, new_vertices| len + new_vertices.len());

        // Allocate enough memory for them all
        created_vertices.reserve(total_new_vertices);

        // Append new vertices
        for mut new_vertices in new_vertices_grouped.into_iter() {
            created_vertices.append(&mut new_vertices);
        }

        GeometryShader {
            mesh,
            uniforms,
            framebuffer,
            indexed_vertices,
            created_vertices,
        }
    }

    /// Clips all triangles along the seven planes that define the view frustum.
    ///
    /// This will most likely generate new triangles for some cases,
    /// but all intermediate uniforms will be interpolated so it shouldn't be noticeable.
    pub fn clip_triangles(self) -> GeometryShader<'a, V, U, K, P> {
        self.triangles(|triangle, uniforms| {
            // TODO
            None
        })
    }

    pub fn finish(self) -> FragmentShader<'a, V, U, K, P, ()> {
        let viewport = self.framebuffer.viewport();

        FragmentShader {
            mesh: self.mesh,
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            indexed_vertices: Arc::new(self.indexed_vertices.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect()),
            created_vertices: Arc::new(self.created_vertices.into_par_iter().map(|vertex| vertex.normalize(viewport)).collect()),
            cull_faces: None,
            blend: (),
        }
    }
}

/// Fragment returned by the fragment shader, which can either be a color
/// value for the pixel or a discard flag to skip that fragment altogether.
#[derive(Debug, Clone, Copy)]
pub enum Fragment<P> where P: Sized + Pixel {
    /// Discard the fragment altogether, as if it was never there.
    Discard,
    /// Desired color for the pixel
    Color(P)
}

/// Describes the style of lines to be drawn in wireframe rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    /// Thin, aliased lines drawn using Bresenham's algorithm
    Thin,
    /// Thin, antialiased line drawn using Xiaolin Wu's algorithm
    ThinAA,
}

impl Default for LineStyle {
    fn default() -> LineStyle { LineStyle::ThinAA }
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
            mesh: self.mesh,
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            indexed_vertices: self.indexed_vertices,
            created_vertices: self.created_vertices,
            cull_faces: self.cull_faces,
        }
    }

    pub fn with_default_blend<B>(self) -> FragmentShader<'a, V, U, K, P, B> where B: Blend<P> + Default {
        self.with_blend(B::default())
    }
}

impl<'a, V, U: 'a, K, P, B> FragmentShader<'a, V, U, K, P, B> where V: Send + Sync,
                                                                    U: Send + Sync,
                                                                    K: Send + Sync + Barycentric,
                                                                    P: Pixel, B: Blend<P> {
    /// Duplicates all references to internal state to return a cloned fragment shader,
    /// which can be used to efficiently render the same geometry with different
    /// rasterization methods in quick succession.
    pub fn duplicate<'b>(&'b mut self) -> FragmentShader<'b, V, U, K, P, B> where 'a: 'b,
                                                                                  B: Clone {
        FragmentShader {
            mesh: self.mesh.clone(),
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            indexed_vertices: self.indexed_vertices.clone(),
            created_vertices: self.created_vertices.clone(),
            cull_faces: self.cull_faces.clone(),
            blend: self.blend.clone()
        }
    }

    /// Cull faces based on winding order. For more information on how and why this works,
    /// check out the documentation for the [`FaceWinding`](../geometry/enum.FaceWinding.html) enum.
    #[inline(always)]
    pub fn cull_faces(&mut self, cull: Option<FaceWinding>) {
        self.cull_faces = cull;
    }

    /// Rasterize the given vertices as triangles.
    ///
    /// Equivalent to `GL_TRIANGLES`
    pub fn triangles<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        // Pull all variables out of self so we can borrow them individually.
        let FragmentShader {
            mesh,
            uniforms,
            framebuffer,
            indexed_vertices,
            created_vertices,
            cull_faces,
            blend
        } = self;

        let (width, height) = (framebuffer.width() as usize,
                               framebuffer.height() as usize);

        // Bounding box for the entire view space
        let bb = (width - 1, height - 1);

        let framebuffer = Mutex::new(framebuffer);

        let triangle_queue: SegQueue<&[u32]> = SegQueue::new();

        // Use chunks of 1024 triangles, giving a balance between granularity and per-chunk performance.
        // For example, a mesh with 4 million triangles will have about 4,000 chunks, and a mesh with 16,000 triangles will have
        // about 16 chunks, so even on small meshes there is a chance for threads to steal the other's work just a little.
        for chunk in mesh.indices.chunks(3 * 1024) {
            triangle_queue.push(chunk);
        }

        let partial_framebuffers = (0..current_num_threads()).into_par_iter().map(|_| -> FrameBuffer<P> {
            let mut framebuffer = framebuffer.lock().unwrap().empty_clone();

            while let Some(chunk) = triangle_queue.try_pop() {
                for triangle in chunk.chunks(3) {
                    // skip incomplete triangles
                    if triangle.len() != 3 { continue; }

                    let ref a = indexed_vertices[triangle[0] as usize];
                    let ref b = indexed_vertices[triangle[1] as usize];
                    let ref c = indexed_vertices[triangle[2] as usize];

                    let XYZW { x: x1, y: y1, .. } = *a.position;
                    let XYZW { x: x2, y: y2, .. } = *b.position;
                    let XYZW { x: x3, y: y3, .. } = *c.position;

                    // do backface culling
                    if let Some(winding) = cull_faces {
                        let a = x1 * y2 + x2 * y3 + x3 * y1 - x2 * y1 - x3 * y2 - x1 * y3;

                        if winding == if a.is_sign_negative() { FaceWinding::Clockwise } else { FaceWinding::CounterClockwise } {
                            continue;
                        }
                    }

                    // calculate determinant
                    let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);

                    // find x bounds for the bounding box
                    let min_x: usize = clamp(x1.min(x2).min(x3) as usize, 0, bb.0);
                    let max_x: usize = clamp(x1.max(x2).max(x3) as usize, 0, bb.0);

                    // find y bounds for the bounding box
                    let min_y: usize = clamp(y1.min(y2).min(y3) as usize, 0, bb.1);
                    let max_y: usize = clamp(y1.max(y2).max(y3) as usize, 0, bb.1);

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
                                            uniforms: Barycentric::interpolate(u, &a.uniforms,
                                                                               v, &b.uniforms,
                                                                               w, &c.uniforms),
                                        }, &*uniforms);

                                        match fragment {
                                            Fragment::Color(c) => {
                                                let fc = unsafe { color.get_unchecked_mut(index) };

                                                *fc = blend.blend_by_depth(z, *fd, c, *fc);
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
                }
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
}