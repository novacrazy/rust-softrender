use super::RasterArguments;

use num_traits::{Float, Zero, NumCast, cast};
use nalgebra::coordinates::XYZW;

use ::color::{Color, ColorAlpha};
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

pub fn rasterize_line<P, V, K, B, F>(args: &RasterArguments<P, V>,
                                     pipeline: &mut P,
                                     blend: &Blend<Pixel<P>>,
                                     fragment_shader: F,
                                     start: &ScreenVertex<V::Scalar, K>,
                                     end: &ScreenVertex<V::Scalar, K>)
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

    use ::geometry::line::liang_barsky_iterative;

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
                    let (xf, yf) = (cast::<_, V::Scalar>(x).unwrap() + NumCast::from(0.5).unwrap(),
                                    cast::<_, V::Scalar>(y).unwrap() + NumCast::from(0.5).unwrap());

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
}


/// Uses Bresenham's algorithm to draw a line.
///
/// [https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)
pub fn draw_line_bresenham<F>(mut x0: i64, mut y0: i64, x1: i64, y1: i64, mut plot: F) where F: FnMut(i64, i64, f64) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = dx + dy;

    loop {
        plot(x0, y0, 1.0);

        if x0 == x1 && y0 == y1 { break; }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            x0 += sx;
        }

        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Uses Xiaolin Wu's algorithm to draw an anti-aliased line.
///
/// [https://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm](https://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm)
///
/// Despite the ropey appearance up close, at a 1 to 1 resolution Xiaolin Wu's technique
/// looks much better than non-antialiased techniques.
pub fn draw_line_xiaolin_wu<F>(mut x0: f64, mut y0: f64, mut x1: f64, mut y1: f64, mut plot: F) where F: FnMut(i64, i64, f64) {
    use std::mem::swap;

    let mut plot_float = |x: f64, y: f64, opacity: f64| {
        plot(x as i64, y as i64, opacity)
    };

    let steep = (y1 - y0).abs() > (x1 - x0).abs();

    if steep {
        swap(&mut x0, &mut y0);
        swap(&mut x1, &mut y1);
    }

    if x0 > x1 {
        swap(&mut x0, &mut x1);
        swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;

    let gradient = if dx < 0.0001 { 1.0 } else { dy / dx };

    let xend = x0.round();
    let yend = y0 + gradient * (xend - x0);

    let xgap = 1.0 - (x0 + 0.5).fract();

    let xpxl1 = xend;
    let ypxl1 = yend.trunc();

    if steep {
        plot_float(ypxl1, xpxl1, (1.0 - yend.fract()) * xgap);
        plot_float(ypxl1 + 1.0, xpxl1, yend.fract() * xgap);
    } else {
        plot_float(xpxl1, ypxl1, (1.0 - yend.fract()) * xgap);
        plot_float(xpxl1, ypxl1 + 1.0, yend.fract() * xgap);
    }

    let mut intery = yend + gradient;

    let xend = x1.round();
    let yend = y1 + gradient * (xend - x1);
    let xgap = (x1 + 0.5).fract();

    let xpxl2 = xend;
    let ypxl2 = yend.trunc();

    if steep {
        plot_float(ypxl2, xpxl2, (1.0 - yend.fract()) * xgap);
        plot_float(ypxl2 + 1.0, xpxl2, yend.fract() * xgap);
    } else {
        plot_float(xpxl2, ypxl2, (1.0 - yend.fract()) * xgap);
        plot_float(xpxl2, ypxl2 + 1.0, yend.fract() * xgap);
    }

    let mut x = xpxl1 + 1.0;

    if steep {
        while x <= (xpxl2 - 1.0) {
            let y = intery.trunc();

            plot_float(y, x, 1.0 - intery.fract());
            plot_float(y + 1.0, x, intery.fract());

            intery += gradient;
            x += 1.0;
        }
    } else {
        while x <= (xpxl2 - 1.0) {
            let y = intery.trunc();

            plot_float(x, y, 1.0 - intery.fract());
            plot_float(x, y + 1.0, intery.fract());

            intery += gradient;
            x += 1.0;
        }
    }
}