use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use rayon::prelude::*;

use num_traits::{Float, One, Zero, NumCast};
use nalgebra::coordinates::XYZW;

use ::error::RenderResult;

use ::numeric::utils::min;
use ::color::Color;
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

pub const DEFAULT_TILE_SIZE: Dimensions = Dimensions { width: 32, height: 32 };

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

    /// Enables drawing antialiased lines for `Line` primitives
    /// primitives using Xiaolin Wu's algorithm,
    /// otherwise Bresenham's Algorithm is used.
    pub fn antialiased_lines(&mut self, enable: bool) {
        self.antialiased_lines = enable;
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

            let mut y = 0;

            while y < dimensions.height {
                let mut x = 0;

                let next_y = min(y + tile_size.height, dimensions.height);

                while x < dimensions.width {
                    let next_x = min(x + tile_size.width, dimensions.width);

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

        /// There is simply no way around this right now. The only reason I'm comfortable doing it is because
        /// all the code using the pipeline is my own and not available to the user.
        ///
        /// Additionally, although the framebuffer access is totally unsafe, the uniforms are requires to be `Send + Sync`, so they
        /// are fine. More or less.
        #[derive(Clone, Copy)]
        struct NeverDoThis<P> { pipeline: *mut P }

        unsafe impl<P> Send for NeverDoThis<P> {}
        unsafe impl<P> Sync for NeverDoThis<P> {}

        let seriously_dont = NeverDoThis { pipeline: pipeline as *mut P };

        tiles.into_par_iter().for_each(|tile: (Coordinate, Coordinate)| {
            println!("Tile {:?}", tile);

            let pipeline: &mut P = unsafe { &mut *seriously_dont.pipeline };

            let rasterize_triangle = |framebuffer: &mut <P as PipelineObject>::Framebuffer,
                                      a: &ScreenVertex<V::Scalar, K>, b: &ScreenVertex<V::Scalar, K>, c: &ScreenVertex<V::Scalar, K>| {
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

                macro_rules! clamp_as_int {
                    ($value:expr, $min:expr, $max:expr) => {{
                        // Store expressions as temp variables to avoid multiple evaluation
                        let value = $value; let min = $min; let max = $max;
                        if value < NumCast::from(min).unwrap() { min } else if value > NumCast::from(max).unwrap() { max } else { NumCast::from(value).unwrap() }
                    }}
                }

                let min = Coordinate::new(clamp_as_int!(x1.min(x2).min(x3), tile.0.x, tile.1.x),
                                          clamp_as_int!(y1.min(y2).min(y3), tile.0.y, tile.1.y));

                let max = Coordinate::new(clamp_as_int!(x1.max(x2).max(x3), tile.0.x, tile.1.x),
                                          clamp_as_int!(y1.max(y2).max(y3), tile.0.y, tile.1.y));

                let stencil_test = pipeline.stencil_config().get_test();
                let stencil_op = pipeline.stencil_config().get_op();

                let mut pixel = min;

                while pixel.y < max.y {
                    pixel.x = min.x;

                    while pixel.x < max.x {
                        let index = pixel.into_index(dimensions);

                        let framebuffer_stencil_value = unsafe { framebuffer.get_stencil_unchecked(index) };

                        // perform stencil test
                        if stencil_test.test(framebuffer_stencil_value, stencil_value) {
                            let new_stencil_value = stencil_op.op(framebuffer_stencil_value, stencil_value);

                            unsafe { framebuffer.set_stencil_unchecked(index, new_stencil_value); }

                            //continue on to fragment shading

                            // Real screen position should be in the center of the pixel.
                            let (x, y) = (<V::Scalar as NumCast>::from(pixel.x).unwrap() + one_half,
                                          <V::Scalar as NumCast>::from(pixel.y).unwrap() + one_half);

                            // calculate barycentric coordinates of the current point
                            let u = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
                            let v = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
                            let w = <V::Scalar as One>::one() - u - v;

                            if !(u < Zero::zero() || v < Zero::zero() || w < Zero::zero()) {
                                // interpolate screen-space position
                                let position = Interpolate::barycentric_interpolate(u, &a.position, v, &b.position, w, &c.position);

                                let z = position.z;

                                if z < Zero::zero() {
                                    let d: DepthAttachment<P::Framebuffer> = Depth::from_scalar(z);

                                    let dt = unsafe { framebuffer.get_depth_unchecked(index) };

                                    if d < dt {
                                        let fragment = fragment_shader(&ScreenVertex {
                                            position,
                                            uniforms: Interpolate::barycentric_interpolate(u, &a.uniforms,
                                                                                           v, &b.uniforms,
                                                                                           w, &c.uniforms),
                                        }, pipeline.uniforms());

                                        match fragment {
                                            Fragment::Discard => (),
                                            Fragment::Color(c) => {
                                                unsafe {
                                                    framebuffer.set_pixel_unchecked(pixel.into_index(dimensions), c);
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

            //render
        });
    }
}