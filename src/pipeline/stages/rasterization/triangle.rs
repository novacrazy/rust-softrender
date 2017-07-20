use super::RasterArguments;

use num_traits::{Float, One, Zero, NumCast, cast};
use nalgebra::coordinates::XYZW;

use ::numeric::utils::min;
use ::color::ColorAlpha;
use ::color::blend::Blend;
use ::pixels::{PixelRead, PixelWrite};
use ::framebuffer::UnsafeFramebuffer;
use ::attachments::depth::Depth;
use ::mesh::{Vertex, Mesh};
use ::geometry::{HasDimensions, Coordinate, ScreenVertex, FaceWinding};
use ::interpolate::Interpolate;

use ::pipeline::PipelineObject;

use ::framebuffer::types::DepthAttachment;
use ::pipeline::types::{PipelineUniforms, Pixel};

use ::pipeline::stages::fragment::Fragment;

pub fn rasterize_triangle<P, V, K, B, F>(args: &RasterArguments<P, V>,
                                         pipeline: &mut P,
                                         blend: B,
                                         fragment_shader: F,
                                         a: &ScreenVertex<V::Scalar, K>,
                                         b: &ScreenVertex<V::Scalar, K>,
                                         c: &ScreenVertex<V::Scalar, K>)
    where P: PipelineObject,
          V: Vertex,
          K: Send + Sync + Interpolate,
          B: Blend<Pixel<P>>,
          F: Fn(&ScreenVertex<V::Scalar, K>, &PipelineUniforms<P>) -> Fragment<Pixel<P>> + Send + Sync {
    let RasterArguments {
        dimensions,
        tile,
        bounds,
        stencil_value,
        stencil_test,
        stencil_op,
        antialiased_lines,
        cull_faces,
    } = *args;

    let (uniforms, framebuffer, _) = pipeline.all_mut();

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
                let (x, y) = (cast::<_, V::Scalar>(pixel.x).unwrap() + NumCast::from(0.5).unwrap(),
                              cast::<_, V::Scalar>(pixel.y).unwrap() + NumCast::from(0.5).unwrap());

                // calculate barycentric coordinates of the current point
                let u = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
                let v = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
                let w = <V::Scalar as One>::one() - u - v;

                // Determine if pixel is even within the triangle
                if !(u < Zero::zero() || v < Zero::zero() || w < Zero::zero()) {
                    // interpolate screen-space position
                    let position = Interpolate::barycentric_interpolate(u, &a.position, v, &b.position, w, &c.position);

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
}