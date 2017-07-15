use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use num_traits::{Float, One, Zero, NumCast, cast};
use nalgebra::coordinates::XYZW;

use ::error::RenderResult;

use ::numeric::utils::min;
use ::color::{Color, ColorAlpha};
use ::color::blend::Blend;
use ::pixels::{PixelRead, PixelWrite};
use ::framebuffer::{UnsafeFramebuffer, Framebuffer};
use ::attachments::depth::Depth;
use ::stencil::StencilConfig;
use ::primitive::Primitive;
use ::mesh::{Vertex, Mesh};
use ::geometry::{Dimensions, HasDimensions, Coordinate, ScreenVertex, FaceWinding};
use ::interpolate::Interpolate;
use ::pipeline::storage::SeparableScreenPrimitiveStorage;

use ::pipeline::PipelineObject;

use ::framebuffer::types::DepthAttachment;
use ::pipeline::types::{PipelineUniforms, Pixel, StencilValue};

pub const DEFAULT_TILE_SIZE: Dimensions = Dimensions { width: 128, height: 128 };

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
pub struct FragmentShader<'a, P: 'a, V: Vertex, T, K, B> where P: PipelineObject {
    pub ( in ::pipeline) pipeline: &'a mut P,
    pub ( in ::pipeline) mesh: Arc<Mesh<V>>,
    pub ( in ::pipeline) indexed_primitive: PhantomData<T>,
    pub ( in ::pipeline) stencil_value: StencilValue<P>,
    pub ( in ::pipeline) indexed_vertices: Arc<Option<Vec<ScreenVertex<V::Scalar, K>>>>,
    pub ( in ::pipeline) generated_primitives: Arc<SeparableScreenPrimitiveStorage<V::Scalar, K>>,
    pub ( in ::pipeline) cull_faces: Option<FaceWinding>,
    pub ( in ::pipeline) blend: B,
    pub ( in ::pipeline) antialiased_lines: bool,
    pub ( in ::pipeline) tile_size: Dimensions,
}

/// Fragment returned by the fragment shader, which can either be a color
/// value for the pixel or a discard flag to skip that fragment altogether.
#[derive(Debug, Clone, Copy)]
pub enum Fragment<C> where C: Color {
    /// Discard the fragment altogether, as if it was never there.
    Discard,
    /// Desired color for the pixel
    Color(C)
}

impl<'a, P: 'a, V, T, K, B> Deref for FragmentShader<'a, P, V, T, K, B>
    where P: PipelineObject, V: Vertex, B: Blend<Pixel<P>> {
    type Target = B;
    fn deref(&self) -> &B { &self.blend }
}

impl<'a, P: 'a, V, T, K, B> DerefMut for FragmentShader<'a, P, V, T, K, B>
    where P: PipelineObject, V: Vertex, B: Blend<Pixel<P>> {
    fn deref_mut(&mut self) -> &mut B { &mut self.blend }
}

impl<'a, P: 'a, V, T, K, B> FragmentShader<'a, P, V, T, K, B> where P: PipelineObject, V: Vertex {
    /// Cull faces based on winding order. For more information on how and why this works,
    /// check out the documentation for the [`FaceWinding`](../geometry/winding/enum.FaceWinding.html) enum.
    pub fn cull_faces(&mut self, cull: Option<FaceWinding>) {
        self.cull_faces = cull;
    }

    pub fn with_faces_culled(self, cull: Option<FaceWinding>) -> Self {
        FragmentShader {
            cull_faces: cull,
            ..self
        }
    }

    /// Enables drawing antialiased lines for `Line` primitives
    /// primitives using Xiaolin Wu's algorithm,
    /// otherwise Bresenham's Algorithm is used.
    pub fn antialiased_lines(&mut self, enable: bool) {
        self.antialiased_lines = enable;
    }

    pub fn with_antialiased_lines(self, enable: bool) -> Self {
        FragmentShader {
            antialiased_lines: enable,
            ..self
        }
    }

    pub fn tile_size(&mut self, tile_size: Dimensions) {
        self.tile_size = tile_size;
    }

    pub fn with_tile_size(self, tile_size: Dimensions) -> Self {
        FragmentShader {
            tile_size,
            ..self
        }
    }

    /// Duplicates all references to internal state to return a cloned fragment shader,
    /// which can be used to efficiently render the same geometry with different
    /// rasterization methods in quick succession.
    #[must_use]
    pub fn duplicate<'b>(&'b mut self) -> FragmentShader<'b, P, V, T, K, B> where 'a: 'b, B: Clone {
        FragmentShader {
            pipeline: self.pipeline,
            mesh: self.mesh.clone(),
            indexed_primitive: PhantomData,
            stencil_value: self.stencil_value,
            indexed_vertices: self.indexed_vertices.clone(),
            generated_primitives: self.generated_primitives.clone(),
            cull_faces: self.cull_faces.clone(),
            blend: self.blend.clone(),
            antialiased_lines: self.antialiased_lines,
            tile_size: self.tile_size,
        }
    }
}

impl<'a, P: 'a, V, T, K, O> FragmentShader<'a, P, V, T, K, O> where P: PipelineObject, V: Vertex {
    #[must_use]
    pub fn with_blend<B>(self, blend: B) -> FragmentShader<'a, P, V, T, K, B>
        where B: Blend<Pixel<P>> {
        FragmentShader {
            pipeline: self.pipeline,
            mesh: self.mesh,
            indexed_primitive: PhantomData,
            stencil_value: self.stencil_value,
            indexed_vertices: self.indexed_vertices,
            generated_primitives: self.generated_primitives,
            cull_faces: self.cull_faces,
            blend: blend,
            antialiased_lines: self.antialiased_lines,
            tile_size: self.tile_size,
        }
    }

    #[must_use]
    pub fn with_default_blend<B>(self) -> FragmentShader<'a, P, V, T, K, B>
        where B: Blend<Pixel<P>> + Default {
        self.with_blend(B::default())
    }
}

impl<'a, P: 'a, V, T, K, B> FragmentShader<'a, P, V, T, K, B> where P: PipelineObject,
                                                                    V: Vertex,
                                                                    T: Primitive,
                                                                    K: Send + Sync + Interpolate,
                                                                    B: Blend<Pixel<P>> {
    pub fn run<S>(self, fragment_shader: S)
        where S: Fn(&ScreenVertex<V::Scalar, K>, &PipelineUniforms<P>) -> Fragment<Pixel<P>> + Send + Sync {
        let FragmentShader {
            pipeline,
            mesh,
            indexed_vertices,
            stencil_value,
            generated_primitives,
            cull_faces,
            blend,
            antialiased_lines,
            tile_size,
            ..
        } = self;

        // Basically constant
        let one_half = <V::Scalar as NumCast>::from(0.5).unwrap();

        let dimensions = pipeline.framebuffer().dimensions();

        let tiles = {
            let mut tiles = Vec::new();

            let xmax = dimensions.width - 1;
            let ymax = dimensions.height - 1;

            let mut y = 0;

            while y < ymax {
                let mut x = 0;

                let next_y = min(y + tile_size.height, ymax);

                while x < xmax {
                    let next_x = min(x + tile_size.width, xmax);

                    tiles.push((
                        Coordinate::new(x, y),
                        Coordinate::new(next_x, next_y)
                    ));

                    x = next_x;
                }

                y = next_y;
            }

            tiles
        };

        // Fetch stencil test and operation before tile loop
        let stencil_test = pipeline.stencil_config().get_test();
        let stencil_op = pipeline.stencil_config().get_op();

        /// There is simply no way around this right now. The only reason I'm comfortable doing it is because
        /// all the code using the pipeline is my own and not available to the user.
        ///
        /// Additionally, although the framebuffer access is totally unsafe, the uniforms are requires to be `Send + Sync`, so they
        /// are fine. More or less.
        #[derive(Clone, Copy)]
        struct NeverDoThis<P> { pipeline: *mut P }

        unsafe impl<P> Send for NeverDoThis<P> {}
        unsafe impl<P> Sync for NeverDoThis<P> {}

        /// Create unsafe mutable point to the pipeline
        let seriously_dont = NeverDoThis { pipeline: pipeline as *mut P };

        ////////////////////////////////////////
        let rasterize_point = |framebuffer: &mut <P as PipelineObject>::Framebuffer,
                               uniforms: &PipelineUniforms<P>,
                               tile: (Coordinate, Coordinate),
                               bounds: ((V::Scalar, V::Scalar), (V::Scalar, V::Scalar)),
                               p: &ScreenVertex<V::Scalar, K>| {
            let XYZW { x, y, z, .. } = *p.position;

            if (bounds.0).0 <= x && x < (bounds.1).0 && (bounds.0).1 <= y && y < (bounds.1).1 {
                let coord = Coordinate::new(cast(x).unwrap(), cast(y).unwrap());

                let index = coord.into_index(dimensions);

                // Get stencil buffer value for this pixel
                let framebuffer_stencil_value = unsafe { framebuffer.get_stencil_unchecked(index) };

                // perform stencil test
                if stencil_test.test(framebuffer_stencil_value, stencil_value) {
                    // Calculate new stencil value
                    let new_stencil_value = stencil_op.op(framebuffer_stencil_value, stencil_value);

                    // Set stencil value for this pixel
                    unsafe { framebuffer.set_stencil_unchecked(index, new_stencil_value); }

                    if z < Zero::zero() {
                        let d: DepthAttachment<P::Framebuffer> = Depth::from_scalar(z);

                        let dt = unsafe { framebuffer.get_depth_unchecked(index) };

                        // Check if point is in front of other geometry
                        if d >= dt {
                            // Perform fragment shading
                            let fragment = fragment_shader(p, &uniforms);

                            match fragment {
                                Fragment::Discard => (),
                                Fragment::Color(c) => {
                                    let p = unsafe { framebuffer.get_pixel_unchecked(index) };

                                    unsafe {
                                        framebuffer.set_pixel_unchecked(index, blend.blend(c, p));
                                        framebuffer.set_depth_unchecked(index, d);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        ////////////////////////////////////////
        let rasterize_line = |framebuffer: &mut <P as PipelineObject>::Framebuffer,
                              uniforms: &PipelineUniforms<P>,
                              tile: (Coordinate, Coordinate),
                              bounds: ((V::Scalar, V::Scalar), (V::Scalar, V::Scalar)),
                              start: &ScreenVertex<V::Scalar, K>,
                              end: &ScreenVertex<V::Scalar, K>| {
            use ::geometry::line::liang_barsky_iterative;
            use super::rasterization::line::{draw_line_bresenham, draw_line_xiaolin_wu};

            let XYZW { x: x1, y: y1, .. } = *start.position;
            let XYZW { x: x2, y: y2, .. } = *end.position;

            if let Some(((x1, y1), (x2, y2))) = liang_barsky_iterative((x1, y1), (x2, y2), bounds) {
                let d = (x1 - x2).hypot(y1 - y2);

                let rasterize_fragment = |x: i64, y: i64, alpha: f64| {
                    if x >= 0 && y >= 0 {
                        let coord = Coordinate::new(x as u32, y as u32);

                        let index = coord.into_index(dimensions);

                        // Get stencil buffer value for this pixel
                        let framebuffer_stencil_value = unsafe { framebuffer.get_stencil_unchecked(index) };

                        // perform stencil test
                        if stencil_test.test(framebuffer_stencil_value, stencil_value) {
                            // Calculate new stencil value
                            let new_stencil_value = stencil_op.op(framebuffer_stencil_value, stencil_value);

                            // Set stencil value for this pixel
                            unsafe { framebuffer.set_stencil_unchecked(index, new_stencil_value); }

                            // Real screen position should be in the center of the pixel.
                            let (xf, yf) = (cast::<_, V::Scalar>(x).unwrap() + one_half,
                                            cast::<_, V::Scalar>(y).unwrap() + one_half);

                            let t = (x1 - xf).hypot(y1 - yf) / d;

                            let position = Interpolate::linear_interpolate(t, &start.position, &end.position);

                            let z = position.z;

                            if z < Zero::zero() {
                                let d: DepthAttachment<P::Framebuffer> = Depth::from_scalar(z);

                                let dt = unsafe { framebuffer.get_depth_unchecked(index) };

                                // Check if point is in front of other geometry
                                if d >= dt {
                                    // Perform fragment shading
                                    let fragment = fragment_shader(&ScreenVertex {
                                        position,
                                        uniforms: Interpolate::linear_interpolate(t, &start.uniforms, &end.uniforms)
                                    }, &uniforms);

                                    match fragment {
                                        Fragment::Discard => (),
                                        Fragment::Color(c) => {
                                            let p = unsafe { framebuffer.get_pixel_unchecked(index) };

                                            unsafe {
                                                framebuffer.set_pixel_unchecked(index, blend.blend(c.mul_alpha(ColorAlpha::from_scalar(alpha)), p));
                                                framebuffer.set_depth_unchecked(index, d);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                };

                if antialiased_lines {
                    draw_line_xiaolin_wu(cast(x1).unwrap(), cast(y1).unwrap(),
                                         cast(x2).unwrap(), cast(y2).unwrap(), rasterize_fragment);
                } else {
                    draw_line_bresenham(cast(x1).unwrap(), cast(y1).unwrap(),
                                        cast(x2).unwrap(), cast(y2).unwrap(), rasterize_fragment)
                }
            }
        };

        ////////////////////////////////////////
        let rasterize_triangle = |framebuffer: &mut <P as PipelineObject>::Framebuffer,
                                  uniforms: &PipelineUniforms<P>,
                                  tile: (Coordinate, Coordinate),
                                  bounds: ((V::Scalar, V::Scalar), (V::Scalar, V::Scalar)),
                                  a: &ScreenVertex<V::Scalar, K>,
                                  b: &ScreenVertex<V::Scalar, K>,
                                  c: &ScreenVertex<V::Scalar, K>| {
            // Dereference/transmute required position components at once
            let XYZW { x: x1, y: y1, .. } = *a.position;
            let XYZW { x: x2, y: y2, .. } = *b.position;
            let XYZW { x: x3, y: y3, .. } = *c.position;

            // do backface culling
            if let Some(winding) = cull_faces {
                // Shoelace algorithm for a triangle
                let a = x1 * y2 + x2 * y3 + x3 * y1 - x2 * y1 - x3 * y2 - x1 * y3;

                if winding == if a.is_sign_negative() { FaceWinding::Clockwise } else { FaceWinding::CounterClockwise } {
                    return;
                }
            }

            // calculate determinant
            let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);

            macro_rules! clamp_as_int {
                    ($value:expr, $min:expr, $max:expr) => {{
                        // Store expressions as temp variables to avoid multiple evaluation
                        let value = $value; let min = $min; let max = $max;
                        if value < cast(min).unwrap() { min } else if value > cast(max).unwrap() { max } else { cast(value).unwrap() }
                    }}
                }

            let min = Coordinate::new(clamp_as_int!(x1.min(x2).min(x3), tile.0.x, tile.1.x),
                                      clamp_as_int!(y1.min(y2).min(y3), tile.0.y, tile.1.y));

            let max = Coordinate::new(clamp_as_int!(x1.max(x2).max(x3), tile.0.x, tile.1.x),
                                      clamp_as_int!(y1.max(y2).max(y3), tile.0.y, tile.1.y));

            let mut pixel = min;

            while pixel.y <= max.y {
                pixel.x = min.x;

                while pixel.x <= max.x {
                    let index = pixel.into_index(dimensions);

                    debug_assert!(index < dimensions.area());

                    // Get stencil buffer value for this pixel
                    let framebuffer_stencil_value = unsafe { framebuffer.get_stencil_unchecked(index) };

                    // perform stencil test
                    if stencil_test.test(framebuffer_stencil_value, stencil_value) {
                        // Calculate new stencil value
                        let new_stencil_value = stencil_op.op(framebuffer_stencil_value, stencil_value);

                        // Set stencil value for this pixel
                        unsafe { framebuffer.set_stencil_unchecked(index, new_stencil_value); }

                        //continue on to fragment shading

                        // Real screen position should be in the center of the pixel.
                        let (x, y) = (cast::<_, V::Scalar>(pixel.x).unwrap() + one_half,
                                      cast::<_, V::Scalar>(pixel.y).unwrap() + one_half);

                        // calculate barycentric coordinates of the current point
                        let u = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
                        let v = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
                        let w = <V::Scalar as One>::one() - u - v;

                        // Determine if pixel is even within the triangle
                        if !(u < Zero::zero() || v < Zero::zero() || w < Zero::zero()) {
                            // interpolate screen-space position
                            let position = Interpolate::barycentric_interpolate(u, &a.position, v, &b.position, w, &c.position);

                            // Dereference/transmute position only once
                            let z = position.z;

                            // Check if point is in front of the screen
                            if z < Zero::zero() {
                                let d: DepthAttachment<P::Framebuffer> = Depth::from_scalar(z);

                                let dt = unsafe { framebuffer.get_depth_unchecked(index) };

                                // Check if point is in front of other geometry
                                if d >= dt {
                                    // Perform fragment shading
                                    let fragment = fragment_shader(&ScreenVertex {
                                        position,
                                        uniforms: Interpolate::barycentric_interpolate(u, &a.uniforms,
                                                                                       v, &b.uniforms,
                                                                                       w, &c.uniforms),
                                    }, uniforms);

                                    match fragment {
                                        Fragment::Discard => (),
                                        Fragment::Color(c) => {
                                            let p = unsafe { framebuffer.get_pixel_unchecked(index) };

                                            unsafe {
                                                framebuffer.set_pixel_unchecked(index, blend.blend(c, p));
                                                framebuffer.set_depth_unchecked(index, d);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    pixel.x += 1;
                }

                pixel.y += 1;
            }
        };

        let (_, _, pool) = pipeline.all_mut();

        let thread_count = pool.thread_count();

        let i = AtomicUsize::new(0);

        pool.scoped(|scope| {
            for _ in 0..thread_count {
                scope.execute(|| {
                    // Get the unsafe mutable reference to the pipeline
                    let pipeline: &mut P = unsafe { &mut *seriously_dont.pipeline };

                    let (uniforms, framebuffer, _) = pipeline.all_mut();

                    loop {
                        let i = i.fetch_add(1, Ordering::Relaxed);

                        if i < tiles.len() {
                            let tile = tiles[i];

                            let bounds = ((cast(tile.0.x).unwrap(), cast(tile.0.y).unwrap()),
                                          (cast(tile.1.x).unwrap(), cast(tile.1.y).unwrap()));

                            if T::is_triangle() {
                                if let Some(ref indexed_vertices) = *indexed_vertices {
                                    for triangle in mesh.indices.chunks(3) {
                                        let a = &indexed_vertices[triangle[0]];
                                        let b = &indexed_vertices[triangle[1]];
                                        let c = &indexed_vertices[triangle[2]];

                                        rasterize_triangle(framebuffer, uniforms, tile, bounds, a, b, c);
                                    }
                                }
                            }

                            for triangle in generated_primitives.tris.chunks(3) {
                                rasterize_triangle(framebuffer, uniforms, tile, bounds, &triangle[0], &triangle[1], &triangle[2]);
                            }

                            if T::is_line() {
                                if let Some(ref indexed_vertices) = *indexed_vertices {
                                    for line in mesh.indices.chunks(2) {
                                        let start = &indexed_vertices[line[0]];
                                        let end = &indexed_vertices[line[1]];

                                        rasterize_line(framebuffer, uniforms, tile, bounds, start, end);
                                    }
                                }
                            }

                            for line in generated_primitives.lines.chunks(2) {
                                rasterize_line(framebuffer, uniforms, tile, bounds, &line[0], &line[1]);
                            }

                            if T::is_point() {
                                if let Some(ref indexed_vertices) = *indexed_vertices {
                                    for index in &mesh.indices {
                                        let point = &indexed_vertices[*index];

                                        rasterize_point(framebuffer, uniforms, tile, bounds, point);
                                    }
                                }
                            }

                            for point in &generated_primitives.points {
                                rasterize_point(framebuffer, uniforms, tile, bounds, point);
                            }
                        } else {
                            break;
                        }
                    }
                });
            }
        });
    }
}