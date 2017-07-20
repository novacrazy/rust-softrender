use super::RasterArguments;

use num_traits::{One, Zero, NumCast, cast};
use nalgebra::coordinates::XYZW;

use ::color::blend::Blend;
use ::pixels::{PixelRead, PixelWrite};
use ::framebuffer::UnsafeFramebuffer;
use ::attachments::depth::Depth;
use ::mesh::Vertex;
use ::geometry::{Coordinate, ScreenVertex, FaceWinding};
use ::interpolate::Interpolate;

use ::pipeline::PipelineObject;

use ::framebuffer::types::DepthAttachment;
use ::pipeline::types::{PipelineUniforms, Pixel};

use ::pipeline::stages::fragment::Fragment;

pub fn rasterize_point<P, V, K, B, F>(args: &RasterArguments<P, V>,
                                      pipeline: &mut P,
                                      blend: B,
                                      fragment_shader: F,
                                      point: &ScreenVertex<V::Scalar, K>)
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

    let XYZW { x, y, z, .. } = *point.position;

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
                    let fragment = fragment_shader(point, &uniforms);

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
}